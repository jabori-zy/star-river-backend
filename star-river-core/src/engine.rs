use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Hash, Eq, PartialEq)]
pub enum EngineName {
    #[strum(serialize = "exchange-engine")]
    ExchangeEngine, // 交易所引擎
    
    #[strum(serialize = "marekt-engine")]
    MarketEngine, // 市场引擎
    
    #[strum(serialize = "indicator-engine")]
    IndicatorEngine, // 指标引擎

    #[strum(serialize = "account-engine")]
    AccountEngine, // 账户引擎

    #[strum(serialize = "strategy-engine")]
    StrategyEngine, // 策略引擎
    
    #[cfg(feature = "paid")]
    #[strum(serialize = "cache-engine")]
    CacheEngine, // 缓存引擎
    

    #[cfg(feature = "paid")]
    #[strum(serialize = "live-strategy-engine")]
    LiveStrategyEngine, // 实时策略引擎
    

}
