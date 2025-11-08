pub use star_river_event::event::{exchange_event::ExchangeEvent, market_event::MarketEvent};

use serde::Serialize;
use strum::{Display, EnumIter};
use event_center_core::event::EventTrait;
use event_center_core::Channel as EventCenterChannel;
use strum::IntoEnumIterator;
use derive_more::From;
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;





#[derive(Debug, Clone, Eq, Hash, PartialEq, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Channel {
    Market,
    Exchange,
    Backtest,
    Account,
    Indicator,
}

impl EventCenterChannel for Channel {
    fn variants() -> Vec<Self> {
        Channel::iter().collect()
    }
}


#[derive(Debug, Clone, Serialize, Display, From)]
#[serde(tag = "channel")]
pub enum Event {
    #[strum(serialize = "exchange")]
    #[serde(rename = "exchange")]
    Exchange(ExchangeEvent),

    #[strum(serialize = "market")]
    #[serde(rename = "market")]
    Market(MarketEvent),

    #[strum(serialize = "backtest")]
    #[serde(rename = "backtest")]
    Backtest(BacktestStrategyEvent),
}

impl EventTrait for Event {

    type C = Channel;

    fn channel(&self) -> &Self::C {
        match self {
            Event::Exchange(_) => &Channel::Exchange,
            Event::Market(_) => &Channel::Market,
            Event::Backtest(_) => &Channel::Backtest,
        }
    }
}
