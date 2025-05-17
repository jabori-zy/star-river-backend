pub mod market_event;
pub mod order_event;
pub mod position_event;
pub mod database_event;
pub mod command;
pub mod response;
pub mod strategy_event;
pub mod indicator_event;
pub mod exchange_event;
pub mod account_event;



use crate::market_event::MarketEvent;
use crate::command::Command;
use crate::exchange_event::ExchangeEvent;
use crate::response::Response;
use crate::strategy_event::StrategyEvent;
use crate::indicator_event::IndicatorEvent;
use crate::order_event::OrderEvent;
use crate::position_event::PositionEvent;
use crate::account_event::AccountEvent;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::{broadcast,mpsc,oneshot};
use std::sync::Arc;
use types::engine::EngineName;
use tokio::sync::Mutex;


pub type EventSender = broadcast::Sender<Event>;
pub type EventReceiver = broadcast::Receiver<Event>;
pub type CommandSender = mpsc::Sender<Command>; // 命令发送器
pub type CommandReceiver = mpsc::Receiver<Command>; // 命令接收器
pub type Responder = oneshot::Sender<Response>; // 响应



#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Channel {
    Market, // 市场通道
    Exchange, // 交易所的原始数据通道
    Trade, // 交易通道
    Order, // 订单通道
    Position, // 仓位通道
    Indicator, // 指标通道
    Strategy, // 策略的数据通过这个通道发送
    Account, // 账户通道
}






impl Channel {
    pub fn get_all_channels() -> Vec<Channel> {
        Channel::iter().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "channel")]
pub enum Event {
    #[strum(serialize = "exchange")]
    #[serde(rename = "exchange")]
    Exchange(ExchangeEvent),

    #[strum(serialize = "market")]
    #[serde(rename = "market")]
    Market(MarketEvent),

    #[strum(serialize = "indicator")]
    #[serde(rename = "indicator")]
    Indicator(IndicatorEvent),

    #[strum(serialize = "strategy")]
    #[serde(rename = "strategy")]
    Strategy(StrategyEvent),

    #[strum(serialize = "order")]
    #[serde(rename = "order")]
    Order(OrderEvent),

    #[strum(serialize = "position")]
    #[serde(rename = "position")]
    Position(PositionEvent),

    #[strum(serialize = "account")]
    #[serde(rename = "account")]
    Account(AccountEvent),
}

impl Event {
    pub fn get_channel(&self) -> Channel {
        match self {
            Event::Market(_) => Channel::Market,
            Event::Indicator(_) => Channel::Indicator,
            Event::Exchange(_) => Channel::Exchange,
            Event::Strategy(_) => Channel::Strategy,
            Event::Order(_) => Channel::Order,
            Event::Position(_) => Channel::Position,
            Event::Account(_) => Channel::Account,
        }
    }
}



#[derive(Debug)]
pub enum EventCenterError {
    ChannelError(String),
    EventSendError(String),
    // 其他错误类型...
}

impl Display for EventCenterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EventCenterError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
            EventCenterError::EventSendError(msg) => write!(f, "Event Send error: {}", msg),
        }
    }
}




#[derive(Debug)]
pub struct EventCenter {
    broadcast_channels: Arc<Mutex<HashMap<Channel, EventSender>>>,
    command_channels: Arc<Mutex<HashMap<EngineName, CommandSender>>>, // 保留每一个引擎的命令发送器
}

impl EventCenter {
    pub fn new() -> Self {
        let event_center = Self {
            broadcast_channels: Arc::new(Mutex::new(HashMap::new())),
            command_channels: Arc::new(Mutex::new(HashMap::new())),
        };
        // event_center.init_channel();
        event_center
    }

    pub async fn init_channel(self) -> Self{
        let channels = Channel::get_all_channels();
        for channel in channels.iter() {
            let (sender, _) = broadcast::channel::<Event>(100);
            let mut broadcast_channels = self.broadcast_channels.lock().await;
            broadcast_channels.insert(channel.clone(), sender);
            tracing::debug!("Event center initialized successfully: {:?}", channel);
        }
        self
    }

    // pub fn get_channels(&self) -> Vec<String> {
    //     self.broadcast_channels.lock().await.keys().map(|k| k.to_string()).collect()
    // }

    pub async fn subscribe(
        &self,
        channel: &Channel,
    ) -> Result<broadcast::Receiver<Event>, EventCenterError> {
        let broadcast_channels = self.broadcast_channels.lock().await;
        let sender = broadcast_channels.get(channel)
            .ok_or(EventCenterError::ChannelError(format!(
                "Channel {} not found",
                channel
            )))?;

        Ok(sender.subscribe())
    }

    pub async fn publish(&self, event: Event) -> Result<(), EventCenterError> {
        let event_channel = event.get_channel();
        let broadcast_channels = self.broadcast_channels.lock().await;
        let sender = broadcast_channels.get(&event_channel)
            .ok_or(EventCenterError::ChannelError(
                "Channel not found".to_string(),
            ))?;
        tracing::debug!("事件发布成功: {:?}", event);

        let _ = sender.send(event);
        Ok(())
    }

    // 获取指定通道的发布者
    // pub fn get_publisher(&self, channel: Channel) -> Result<broadcast::Sender<Event>, EventCenterError> {
    //     let sender = self
    //         .channels
    //         .get(&channel)
    //         .ok_or(EventCenterError::ChannelError(format!("Channel {} not found", channel)))?;
    //     Ok(sender.clone())
    // }

    pub fn get_event_publisher(&self) -> EventPublisher {
        // 只克隆 Arc，非常轻量
        EventPublisher::new(self.broadcast_channels.clone())
    }

    pub fn get_command_publisher(&self) -> CommandPublisher {
        // 只克隆 Arc，非常轻量
        CommandPublisher::new(self.command_channels.clone())
    }

    // 设置引擎的命令发送器
    pub async fn set_engine_command_sender(&mut self, engine_name: EngineName, sender: CommandSender) {
        let mut command_channels = self.command_channels.lock().await;
        command_channels.insert(engine_name, sender);
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
        let sender = channels.get(&channel)
            .ok_or_else(|| EventCenterError::ChannelError(format!("Channel {} not found", channel)))?;

        match event.clone() {
            Event::Exchange(exchange_event) => {
                // tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, exchange_event);
            }
            _ => {
                // tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, event);
            }
        }
        
        sender.send(event).map_err(|e| 
            EventCenterError::EventSendError(format!("Failed to send event: {}", e))
        )?;

        
        Ok(())
    }
}


#[derive(Clone, Debug)]
pub struct CommandPublisher {
    channels: Arc<Mutex<HashMap<EngineName, CommandSender>>>,
}

impl CommandPublisher {
    pub fn new(channels: Arc<Mutex<HashMap<EngineName, CommandSender>>>) -> Self {
        Self { channels }
    }

    pub async fn send(&self, command: Command) -> Result<(), EventCenterError> {
        let engine_name = command.get_engine_name();
        let channels = self.channels.lock().await;
        let sender = channels.get(&engine_name)
            .ok_or(EventCenterError::ChannelError(format!("Engine name {} not found", engine_name)))?;
        sender.send(command).await.map_err(|e| 
            EventCenterError::EventSendError(format!("Failed to send command: {}", e))
        )?;
        Ok(())
    }
    
}

