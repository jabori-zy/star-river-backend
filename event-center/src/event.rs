pub mod account_event;
pub mod database_event;
pub mod exchange_event;
pub mod indicator_event;
pub mod market_event;
pub mod node_event;
pub mod order_event;
pub mod position_event;
pub mod strategy_event;

pub use account_event::AccountEvent;
pub use exchange_event::ExchangeEvent;
pub use indicator_event::IndicatorEvent;
pub use market_event::MarketEvent;
pub use order_event::OrderEvent;
pub use position_event::PositionEvent;
pub use strategy_event::StrategyEvent;

use crate::Channel;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::broadcast;

pub type EventSender = broadcast::Sender<Event>;
pub type EventReceiver = broadcast::Receiver<Event>;

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
