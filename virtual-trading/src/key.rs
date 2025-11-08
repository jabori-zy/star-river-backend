use star_river_core::exchange::Exchange;
use star_river_core::kline::KlineInterval;
use serde::{Deserialize, Serialize};

/// K线键 - 用于标识唯一的K线数据
/// 
/// 这个结构体是 `key::KlineKey` 的简化版本，移除了对 `strategy-core` 的依赖
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KlineKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl KlineKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, start_time: Option<String>, end_time: Option<String>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            start_time,
            end_time,
        }
    }
}

