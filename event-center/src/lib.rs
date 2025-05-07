pub mod market_event;
pub mod order_event;
pub mod position_event;
pub mod database_event;
pub mod command_event;
pub mod response_event;
pub mod strategy_event;
pub mod indicator_event;
pub mod exchange_event;
pub mod account_event;



use crate::market_event::MarketEvent;
use crate::command_event::CommandEvent;
use crate::exchange_event::ExchangeEvent;
use crate::response_event::ResponseEvent;
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
use tokio::sync::broadcast;

use std::sync::Arc;




#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Channel {
    Market, // 市场通道
    Exchange, // 交易所的原始数据通道
    Trade, // 交易通道
    Order, // 订单通道
    Position, // 仓位通道
    Indicator, // 指标通道
    Strategy, // 策略的信息通过这个通道发送
    Command, // 命令通道
    Response, // 响应通道
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

    #[strum(serialize = "command")]
    #[serde(rename = "command")]
    Command(CommandEvent),

    #[strum(serialize = "response")]
    #[serde(rename = "response")]
    Response(ResponseEvent),

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
            Event::Command(_) => Channel::Command,
            Event::Exchange(_) => Channel::Exchange,
            Event::Response(_) => Channel::Response,
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

#[derive(Clone)]
pub struct EventCenter {
    channels: HashMap<Channel, broadcast::Sender<Event>>,
}

impl EventCenter {
    pub fn new() -> Self {
        let mut event_center = Self {
            channels: HashMap::new(),
        };
        event_center.init_channel();
        event_center
    }

    fn init_channel(&mut self) {
        let channels = Channel::get_all_channels();
        for channel in channels.iter() {
            let (sender, _) = broadcast::channel::<Event>(100);
            self.channels.insert(channel.clone(), sender);
            tracing::debug!("Event center initialized successfully: {:?}", channel);
        }
    }

    pub fn get_channels(&self) -> Vec<String> {
        self.channels.keys().map(|k| k.to_string()).collect()
    }

    pub fn subscribe(
        &self,
        channel: &Channel,
    ) -> Result<broadcast::Receiver<Event>, EventCenterError> {
        let sender = self
            .channels
            .get(channel)
            .ok_or(EventCenterError::ChannelError(format!(
                "Channel {} not found",
                channel
            )))?;

        Ok(sender.subscribe())
    }

    pub fn publish(&self, event: Event) -> Result<(), EventCenterError> {
        let event_channel = event.get_channel();
        let sender = self
            .channels
            .get(&event_channel)
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
        EventPublisher::new(self.channels.clone())
    }


}


#[derive(Clone, Debug)]
pub struct EventPublisher {
    channels: Arc<HashMap<Channel, broadcast::Sender<Event>>>,
}

impl EventPublisher {
    pub fn new(channels: HashMap<Channel, broadcast::Sender<Event>>) -> Self {
        Self { 
            channels: Arc::new(channels) 
        }
    }

    pub fn publish(&self, event: Event) -> Result<(), EventCenterError> {
        let channel = event.get_channel();
        // 使用 get 而不是 get_channel() 来避免额外的匹配开销
        let sender = self.channels.get(&channel)
            .ok_or_else(|| EventCenterError::ChannelError(format!("Channel {} not found", channel)))?;

        match event.clone() {
            Event::Market(market_event) => {
                // tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, market_event);
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

