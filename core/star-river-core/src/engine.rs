use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
// use event_center::event::Event;
// use event_center::communication::engine::EngineCommand;

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Hash, Eq, PartialEq, EnumIter)]
pub enum EngineName {
    #[strum(serialize = "exchange-engine")]
    ExchangeEngine, // Exchange engine

    #[strum(serialize = "market-engine")]
    MarketEngine, // Market engine

    #[strum(serialize = "indicator-engine")]
    IndicatorEngine, // Indicator engine

    // #[strum(serialize = "account-engine")]
    // AccountEngine, // Account engine

    // #[strum(serialize = "cache-engine")]
    // CacheEngine, // Cache engine
    #[strum(serialize = "backtest-engine")]
    BacktestEngine, // Backtest engine
                    // #[strum(serialize = "live-strategy-engine")]
                    // LiveStrategyEngine, // Live strategy engine
}

// #[async_trait]
// pub trait EngineContext {
//     async fn handle_event(&mut self, event: Event);
//     async fn handle_command(&mut self, command: EngineCommand);
// }
