use crate::market::{Exchange, KlineInterval};
use crate::indicator::IndicatorConfig;
use serde::{Deserialize, Serialize};
use super::CacheKeyTrait;
use super::CacheKey;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlineCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl From<KlineCacheKey> for CacheKey {
    fn from(kline_cache_key: KlineCacheKey) -> Self {
        CacheKey::Kline(kline_cache_key)
    }
}

impl KlineCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self { exchange, symbol, interval }
    }
}

impl CacheKeyTrait for KlineCacheKey {
    fn get_key(&self) -> String {
        format!("kline|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval)
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndicatorCacheKey {
    pub kline_cache_key: KlineCacheKey,
    pub indicator_config: IndicatorConfig,
}


impl From<IndicatorCacheKey> for CacheKey {
    fn from(indicator_cache_key: IndicatorCacheKey) -> Self {
        CacheKey::Indicator(indicator_cache_key)
    }
}

impl IndicatorCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, indicator_config: IndicatorConfig) -> Self {
        let kline_cache_key = KlineCacheKey::new(exchange, symbol, interval);
        Self { kline_cache_key, indicator_config }
    }

    pub fn get_indicator(&self) -> IndicatorConfig {
        self.indicator_config.clone()
    }
}

impl CacheKeyTrait for IndicatorCacheKey {
    fn get_key(&self) -> String {
        format!("indicator|{}|{}|{}|{}", self.kline_cache_key.exchange.to_string(), self.kline_cache_key.symbol, self.kline_cache_key.interval, self.indicator_config.to_string())
    }
    fn get_exchange(&self) -> Exchange {
        self.kline_cache_key.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.kline_cache_key.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.kline_cache_key.interval.clone()
    }
}