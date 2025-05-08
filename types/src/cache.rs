use crate::market::Exchange;
use serde::{Deserialize, Serialize};
use crate::indicator::IndicatorConfig;
use crate::market::KlineInterval;
use std::hash::Hash;
use std::fmt::Debug;
use crate::market::Kline;

pub trait CacheKey{
    fn get_key(&self) -> String;
}

pub enum NewCacheKey {
    Kline(KlineCacheKey),
    Indicator(IndicatorCacheKey),
}

pub enum CacheValue {
    Kline(Kline),
    Indicator(IndicatorConfig),
}

impl NewCacheKey {
    pub fn get_key(&self) -> String {
        match self {
            NewCacheKey::Kline(key) => key.get_key(),
            NewCacheKey::Indicator(key) => key.get_key(),
        }
    }
}

pub trait CacheData: Debug {}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlineCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl KlineCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self { exchange, symbol, interval }
    }
}

impl CacheKey for KlineCacheKey {
    fn get_key(&self) -> String {
        format!("{}:{}:{}", self.exchange, self.symbol, self.interval)
    }
}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndicatorCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
}

impl CacheKey for IndicatorCacheKey {
    fn get_key(&self) -> String {
        format!("{}:{}:{}:{}", self.exchange, self.symbol, self.interval, self.indicator)
    }
}




// 缓存管理器



