pub mod communication;
pub mod error;
pub mod event;

use std::{collections::HashMap, sync::Arc};

pub use communication::{CommandTarget, Target};
pub use event::{Channel, Event, EventBase, EventTrait};
use tokio::sync::{Mutex, broadcast, mpsc};

/// 泛型事件中心基础结构
///
/// 泛型参数：
/// - `EC`: Event Channel 类型（事件通道的键类型）
/// - `CC`: Command Target 类型（命令通道的键类型）
/// - `E`: Event 类型（事件类型，需要实现 EventTrait）
/// - `T`: Command 类型（命令类型，需要实现 CommandTarget）
pub struct EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>, // broadcast channel 要求 Event 类型实现 Clone
    T: Target<T = CC>,     // mpsc channel 要求 Command 类型实现 CommandTrait
{
    event_channels: HashMap<EC, broadcast::Sender<E>>,
    command_targets: HashMap<CC, (mpsc::Sender<T>, Arc<Mutex<mpsc::Receiver<T>>>)>,
    _black_hole: HashMap<EC, broadcast::Receiver<E>>,
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    pub fn new() -> Self {
        Self {
            event_channels: HashMap::new(),
            command_targets: HashMap::new(),
            _black_hole: HashMap::new(),
        }
    }

    pub fn init_event_channels_from_iter(mut self, channels: impl IntoIterator<Item = EC>, buffer_size: usize) -> Self {
        for channel in channels {
            let (tx, rx) = broadcast::channel::<E>(buffer_size);
            self.event_channels.insert(channel.clone(), tx);
            self._black_hole.insert(channel, rx);
        }
        self
    }

    /// 从任意迭代器初始化命令通道
    ///
    /// # Arguments
    /// * `channels` - 命令通道的迭代器
    /// * `buffer_size` - 每个通道的缓冲区大小
    pub fn init_command_channels_from_iter(mut self, channels: impl IntoIterator<Item = CC>, buffer_size: usize) -> Self {
        for channel in channels {
            let (tx, rx) = mpsc::channel::<T>(buffer_size);
            self.command_targets.insert(channel, (tx, Arc::new(Mutex::new(rx))));
        }
        self
    }

    pub fn add_channel(mut self, channel: EC, buffer_size: usize) -> Self {
        let (tx, rx) = broadcast::channel::<E>(buffer_size);
        self.event_channels.insert(channel.clone(), tx);
        self._black_hole.insert(channel, rx);
        self
    }

    // 订阅指定通道的事件
    pub fn subscribe(&self, channel: &EC) -> Option<broadcast::Receiver<E>> {
        self.event_channels.get(channel).map(|sender| sender.subscribe())
    }

    // 发布事件到指定通道
    // 返回 Result，Err 中包含发送失败时的错误（接收者数量）
    pub fn publish(&self, event: E) -> Result<usize, broadcast::error::SendError<E>> {
        let channel = event.channel();
        match self.event_channels.get(channel) {
            Some(sender) => sender.send(event),
            None => Ok(0), // 通道不存在，返回 0 个接收者
        }
    }

    pub async fn send_command(&self, command: T) -> Result<(), mpsc::error::SendError<T>> {
        let target = command.target();
        match self.command_targets.get(target) {
            Some((sender, _)) => {
                sender.send(command).await?;
                Ok(())
            }
            None => Err(mpsc::error::SendError(command)), // 通道不存在，返回错误
        }
    }

    pub fn command_sender(&self, target: &CC) -> Option<mpsc::Sender<T>> {
        self.command_targets.get(target).map(|(sender, _)| sender.clone())
    }

    pub fn command_receiver(&self, target: &CC) -> Option<Arc<Mutex<mpsc::Receiver<T>>>> {
        self.command_targets.get(target).map(|(_, receiver)| receiver.clone())
    }
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    pub fn init_event_channels(self, buffer_size: usize) -> Self {
        self.init_event_channels_from_iter(EC::variants(), buffer_size)
    }
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    /// 自动初始化所有命令通道（适合枚举类型）
    ///
    /// 要求 `CC` 类型实现 `EnumerableChannel` trait
    ///
    /// # Arguments
    /// * `buffer_size` - 每个通道的缓冲区大小
    pub fn init_command_channels(self, buffer_size: usize) -> Self {
        self.init_command_channels_from_iter(CC::variants(), buffer_size)
    }
}
