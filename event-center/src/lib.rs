// pub mod response;
pub mod communication;
pub mod event;
pub mod error;
// pub mod singleton;

// use crate::communication::engine::EngineCommand;
use crate::event::strategy_event::StrategyEvent;

// pub use singleton::EventCenterSingleton;

// use crate::communication::engine::{EngineCommandReceiver, EngineCommandSender};
use crate::error::*;
use serde::{Deserialize, Serialize};
// use star_river_core::engine::EngineName;
use std::collections::HashMap;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::event::{Event, EventReceiver, EventSender};

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Channel {
    Market,    // 市场通道
    Exchange,  // 交易所的原始数据通道
    Trade,     // 交易通道
    Order,     // 订单通道
    Position,  // 仓位通道
    Indicator, // 指标通道
    Strategy,  // 策略的数据通过这个通道发送
    Account,   // 账户通道
}

impl Channel {
    pub fn get_all_channels() -> Vec<Channel> {
        Channel::iter().collect()
    }
}

#[derive(Debug)]
pub struct EventCenter {
    pub(crate) broadcast_channels: HashMap<Channel, EventSender>,
    pub(crate) command_channels: HashMap<EngineName, (EngineCommandSender, Arc<Mutex<EngineCommandReceiver>>)>, // 成对保存发送器和接收器
    black_hole: HashMap<Channel, EventReceiver>, // 黑洞通道，用于接收所有事件，但不进行处理
}

impl EventCenter {
    pub fn new() -> Self {
        let event_center = Self {
            broadcast_channels: HashMap::new(),
            command_channels: HashMap::new(),
            black_hole: HashMap::new(),
        };
        // event_center.init_channel();
        event_center
    }

    pub async fn init_channel(mut self) -> Self {
        let channels = Channel::get_all_channels();
        for channel in channels.iter() {
            let (tx, rx) = broadcast::channel::<Event>(100);
            self.broadcast_channels.insert(channel.clone(), tx);
            self.black_hole.insert(channel.clone(), rx); // 插入黑洞通道，用于接收所有事件，但不进行处理
            tracing::debug!("Event center initialized successfully: {:?}", channel);
        }

        // 初始化所有引擎的命令通道
        self.init_command_channels();

        self
    }

    // 新增：初始化所有引擎的命令通道
    fn init_command_channels(&mut self) {
        let engines = vec![
            EngineName::ExchangeEngine,
            EngineName::MarketEngine,
            EngineName::IndicatorEngine,
            EngineName::BacktestEngine,
            // EngineName::AccountEngine,
        ];
        
        
        // engines.push(EngineName::CacheEngine);

        for engine_name in engines.iter() {
            let (tx, rx) = mpsc::channel::<EngineCommand>(100);
            // 成对保存发送器和接收器
            self.command_channels.insert(engine_name.clone(), (tx, Arc::new(Mutex::new(rx))));
            tracing::debug!("Command channel initialized for engine: {:?}", engine_name);
        }
    }

    // pub fn get_channels(&self) -> Vec<String> {
    //     self.broadcast_channels.lock().await.keys().map(|k| k.to_string()).collect()
    // }

    pub async fn subscribe(&self, channel: &Channel) -> Result<broadcast::Receiver<Event>, EventCenterError> {
        let sender = self.broadcast_channels.get(channel).ok_or(
            ChannelNotFoundSnafu {
                channel: channel.to_string(),
            }
            .build(),
        )?;

        Ok(sender.subscribe())
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventCenterError> {
        let event_channel = event.get_channel();
        let sender = self.broadcast_channels.get(&event_channel).ok_or(
            ChannelNotInitializedSnafu {
                channel: event_channel.to_string(),
            }
            .build(),
        )?;

        tracing::debug!("事件发布成功: {:?}", event);

        sender.send(event)?;
        Ok(())
    }

    pub fn get_event_publisher(&self) -> EventPublisher {
        // 现在需要包装成 Arc<Mutex<>> 来保持兼容性
        EventPublisher::new(Arc::new(Mutex::new(self.broadcast_channels.clone())))
    }

    // pub fn get_command_publisher(&self) -> CommandPublisher {
    //     // 提取所有的 CommandSender
    //     let command_senders: HashMap<EngineName, EngineCommandSender> = self
    //         .command_channels
    //         .iter()
    //         .map(|(name, (sender, _receiver))| (name.clone(), sender.clone()))
    //         .collect();

    //     CommandPublisher::new(Arc::new(Mutex::new(command_senders)))
    // }

    // 获取指定引擎的命令接收器
    pub async fn get_command_receiver(&self, engine_name: &EngineName) -> Result<Arc<Mutex<EngineCommandReceiver>>, EventCenterError> {
        self.command_channels
            .get(engine_name)
            .map(|(_sender, receiver)| receiver.clone())
            .ok_or(
                EngineCommandReceiverNotFoundSnafu {
                    engine_name: engine_name.to_string(),
                }
                .build(),
            )
    }

    // 获取指定引擎的命令发送器
    pub fn get_command_sender(&self, engine_name: EngineName) -> Result<EngineCommandSender, EventCenterError> {
        self.command_channels
            .get(&engine_name)
            .map(|(sender, _receiver)| sender.clone())
            .ok_or(
                EngineCommandSenderNotFoundSnafu {
                    engine_name: engine_name.to_string(),
                }
                .build(),
            )
    }
}

#[derive(Clone, Debug)]
pub struct EventPublisher {
    channels: Arc<Mutex<HashMap<Channel, broadcast::Sender<Event>>>>,
}

impl EventPublisher {
    pub fn new(channels: Arc<Mutex<HashMap<Channel, broadcast::Sender<Event>>>>) -> Self {
        Self { channels }
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventCenterError> {
        let channel = event.get_channel();
        // 使用 get 而不是 get_channel() 来避免额外的匹配开销
        let channels = self.channels.lock().await;
        let sender = channels.get(&channel).ok_or(
            ChannelNotFoundSnafu {
                channel: channel.to_string(),
            }
            .build(),
        )?;

        // match event.clone() {
        //     Event::Strategy(strategy_event) => {
        //         // tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, strategy_event);
        //     }
        //     _ => {
        //         // tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, event);
        //     }
        // }

        sender.send(event)?;

        Ok(())
    }
}
