use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
// use event_center::event::Event;
// use event_center::communication::engine::EngineCommand;

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Hash, Eq, PartialEq, EnumIter)]
pub enum EngineName {
    #[strum(serialize = "exchange-engine")]
    ExchangeEngine, // 交易所引擎

    #[strum(serialize = "marekt-engine")]
    MarketEngine, // 市场引擎

    #[strum(serialize = "indicator-engine")]
    IndicatorEngine, // 指标引擎

    // #[strum(serialize = "account-engine")]
    // AccountEngine, // 账户引擎

    // #[strum(serialize = "cache-engine")]
    // CacheEngine, // 缓存引擎
    #[strum(serialize = "backtest-engine")]
    BacktestEngine, // 回测引擎
                    // #[strum(serialize = "live-strategy-engine")]
                    // LiveStrategyEngine, // 实时策略引擎
}

// #[async_trait]
// pub trait EngineContext {
//     async fn handle_event(&mut self, event: Event);
//     async fn handle_command(&mut self, command: EngineCommand);
// }
