use std::sync::{Arc, OnceLock};

use event_center_core::error::{
    ChannelNotFoundSnafu, CmdSendFailedSnafu, CommandReceiverNotFoundSnafu, CommandSenderNotFoundSnafu, EventCenterError,
    EventSendFailedSnafu, InstanceAlreadyInitSnafu, InstanceNotInitSnafu,
};
use snafu::{IntoError, OptionExt};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};

use crate::{Channel, CommandTargetEngine, EngineCommand, EventCenter, event::Event};

static EVENT_CENTER_INSTANCE: OnceLock<Arc<RwLock<EventCenter>>> = OnceLock::new();

pub struct EventCenterSingleton;

impl EventCenterSingleton {
    /// 使用默认配置初始化事件中心单例
    pub fn init() -> Result<(), EventCenterError> {
        let event_center = Arc::new(RwLock::new(EventCenter::new().init_with_default()));

        EVENT_CENTER_INSTANCE
            .set(event_center)
            .map_err(|_| InstanceAlreadyInitSnafu {}.build())?;
        Ok(())
    }

    /// 使用自定义缓冲区大小初始化事件中心单例
    ///
    /// # Arguments
    /// * `event_buffer_size` - 事件通道的缓冲区大小
    /// * `command_buffer_size` - 命令通道的缓冲区大小
    pub fn init_with_size(event_buffer_size: usize, command_buffer_size: usize) -> Result<(), EventCenterError> {
        let event_center = Arc::new(RwLock::new(
            EventCenter::new().init_channels(event_buffer_size, command_buffer_size),
        ));

        EVENT_CENTER_INSTANCE
            .set(event_center)
            .map_err(|_| InstanceAlreadyInitSnafu {}.build())?;
        Ok(())
    }

    /// 检查事件中心是否已初始化
    pub fn is_initialized() -> bool {
        EVENT_CENTER_INSTANCE.get().is_some()
    }

    pub async fn get_instance() -> Result<&'static Arc<RwLock<EventCenter>>, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE.get().ok_or_else(|| InstanceNotInitSnafu {}.build())?;
        Ok(instance)
    }

    pub async fn subscribe(channel: &Channel) -> Result<broadcast::Receiver<Event>, EventCenterError> {
        let instance = Self::get_instance().await?;
        let instance_guard = instance.read().await;
        let receiver = instance_guard.subscribe(channel).context(ChannelNotFoundSnafu {
            channel: channel.to_string(),
        })?;
        Ok(receiver)
    }

    pub async fn publish(event: Event) -> Result<(), EventCenterError> {
        let instance = Self::get_instance().await?;
        let instance_guard = instance.read().await;
        instance_guard
            .publish(event)
            .map_err(|e| EventSendFailedSnafu {}.into_error(Arc::new(e)))?;
        Ok(())
    }

    pub async fn send_command(command: EngineCommand) -> Result<(), EventCenterError> {
        let instance = Self::get_instance().await?;
        let instance_guard = instance.read().await;
        instance_guard
            .send_command(command)
            .await
            .map_err(|e| CmdSendFailedSnafu {}.into_error(Arc::new(e)))?;
        Ok(())
    }

    pub async fn command_sender(target: &CommandTargetEngine) -> Result<mpsc::Sender<EngineCommand>, EventCenterError> {
        let instance = Self::get_instance().await?;
        let instance_guard = instance.read().await;
        let sender = instance_guard.command_sender(target).context(CommandSenderNotFoundSnafu {
            target: target.to_string(),
        })?;
        Ok(sender)
    }

    pub async fn command_receiver(target: &CommandTargetEngine) -> Result<Arc<Mutex<mpsc::Receiver<EngineCommand>>>, EventCenterError> {
        let instance = Self::get_instance().await?;
        let instance_guard = instance.read().await;
        let receiver = instance_guard.command_receiver(target).context(CommandReceiverNotFoundSnafu {
            target: target.to_string(),
        })?;
        Ok(receiver)
    }
}
