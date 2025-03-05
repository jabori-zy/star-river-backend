pub mod market_event;
pub mod command_event;
pub mod indicator_event;
pub mod exchange_event;
pub mod response_event;

use market_event::MarketEvent;
use command_event::CommandEvent;
use exchange_event::ExchangeEvent;
use response_event::ResponseEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::broadcast;
use indicator_event::IndicatorEvent;
use std::sync::Arc;




#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Channel {
    Market,
    Exchange,
    Trade,
    Order,
    Position,
    Indicator,
    Command,
    Response,
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
}

impl Event {
    pub fn get_channel(&self) -> Channel {
        match self {
            Event::Market(_) => Channel::Market,
            Event::Indicator(_) => Channel::Indicator,
            Event::Command(_) => Channel::Command,
            Event::Exchange(_) => Channel::Exchange,
            Event::Response(_) => Channel::Response,
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
        channel: Channel,
    ) -> Result<broadcast::Receiver<Event>, EventCenterError> {
        let sender = self
            .channels
            .get(&channel)
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
    pub fn get_publisher(&self, channel: Channel) -> Result<broadcast::Sender<Event>, EventCenterError> {
        let sender = self
            .channels
            .get(&channel)
            .ok_or(EventCenterError::ChannelError(format!("Channel {} not found", channel)))?;
        Ok(sender.clone())
    }

    pub fn get_publisher1(&self) -> EventPublisher {
        // 只克隆 Arc，非常轻量
        EventPublisher::new(self.channels.clone())
    }

    pub fn get_receiver(&self) -> EventReceiver {
        EventReceiver::new(self)
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

        tracing::debug!("发布事件: 事件通道: {:?}, 事件: {:?}", channel, event);
        
        sender.send(event).map_err(|e| 
            EventCenterError::EventSendError(format!("Failed to send event: {}", e))
        )?;

        
        Ok(())
    }
}

#[derive(Debug)]
pub struct EventReceiver {
    pub market: broadcast::Receiver<Event>,
    pub exchange: broadcast::Receiver<Event>,
    pub trade: broadcast::Receiver<Event>,
    pub order: broadcast::Receiver<Event>,
    pub position: broadcast::Receiver<Event>,
    pub indicator: broadcast::Receiver<Event>,
    pub command: broadcast::Receiver<Event>,
    pub response: broadcast::Receiver<Event>,
}

impl Clone for EventReceiver {
    fn clone(&self) -> Self {
        Self {
            market: self.market.resubscribe(),
            exchange: self.exchange.resubscribe(),
            trade: self.trade.resubscribe(),
            order: self.order.resubscribe(),
            position: self.position.resubscribe(),
            indicator: self.indicator.resubscribe(),
            command: self.command.resubscribe(),
            response: self.response.resubscribe(),
        }
    }
}

impl EventReceiver {
    pub fn new(event_center: &EventCenter) -> Self {
        Self {
            market: event_center.subscribe(Channel::Market).unwrap(),
            exchange: event_center.subscribe(Channel::Exchange).unwrap(),
            trade: event_center.subscribe(Channel::Trade).unwrap(),
            order: event_center.subscribe(Channel::Order).unwrap(),
            position: event_center.subscribe(Channel::Position).unwrap(),
            indicator: event_center.subscribe(Channel::Indicator).unwrap(),
            command: event_center.subscribe(Channel::Command).unwrap(),
            response: event_center.subscribe(Channel::Response).unwrap(),
        }
    }
}


