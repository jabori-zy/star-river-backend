pub mod market_event;
pub mod command_event;
pub mod indicator_event;
pub mod exchange_event;

use market_event::MarketEvent;
use command_event::CommandEvent;
use exchange_event::ExchangeEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::broadcast;
use indicator_event::IndicatorEvent;




#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Channel {
    Market,
    Trade,
    Order,
    Position,
    Indicator,
    Command,
    Exchange,
}

impl Channel {
    pub fn get_all_channels() -> Vec<Channel> {
        Channel::iter().collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
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
}

impl Event {
    pub fn get_channel(&self) -> Channel {
        match self {
            Event::Market(_) => Channel::Market,
            Event::Indicator(_) => Channel::Indicator,
            Event::Command(_) => Channel::Command,
            Event::Exchange(_) => Channel::Exchange,
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
}










