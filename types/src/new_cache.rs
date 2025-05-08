use crate::market::Exchange;
use serde::{Deserialize, Serialize};
use crate::indicator::IndicatorConfig;
use crate::market::KlineInterval;
use std::hash::Hash;
use std::fmt::Debug;
use crate::market::Kline;
use crate::indicator::sma::SMA;
use crate::indicator::bbands::BBands;

pub trait CacheKeyTrait{
    fn get_key(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CacheKey {
    Kline(KlineCacheKey),
    Indicator(IndicatorCacheKey),
}

impl CacheKey {
    pub fn get_key(&self) -> String {
        match self {
            CacheKey::Kline(key) => key.get_key(),
            CacheKey::Indicator(key) => key.get_key(),
        }
    }

    pub fn get_exchange(&self) -> Exchange {
        match self {
            CacheKey::Kline(key) => key.exchange.clone(),
            CacheKey::Indicator(key) => key.exchange.clone(),
        }
    }

    pub fn get_symbol(&self) -> String {
        match self {
            CacheKey::Kline(key) => key.symbol.clone(),
            CacheKey::Indicator(key) => key.symbol.clone(),
        }
    }

    pub fn get_interval(&self) -> KlineInterval {
        match self {
            CacheKey::Kline(key) => key.interval.clone(),
            CacheKey::Indicator(key) => key.interval.clone(),
        }
    }
    
}


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

impl CacheKeyTrait for KlineCacheKey {
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

impl CacheKeyTrait for IndicatorCacheKey {
    fn get_key(&self) -> String {
        format!("{}:{}:{}:{}", self.exchange, self.symbol, self.interval, self.indicator)
    }
}





pub trait CacheValueTrait {
    fn to_cache_value(&self) -> CacheValue;
    fn to_json(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn get_timestamp(&self) -> i64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheValue {
    Kline(Kline),
    SMA(SMA),
    BBands(BBands),
}

impl CacheValue {
    
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            CacheValue::Kline(value) => value.to_json(),
            CacheValue::SMA(value) => value.to_json(),
            CacheValue::BBands(value) => value.to_json(),
        }
    }

    pub fn to_list(&self) -> Vec<f64> {
        match self {
            CacheValue::Kline(value) => value.to_list(),
            CacheValue::SMA(value) => value.to_list(),
            CacheValue::BBands(value) => value.to_list(),
        }
    }

    pub fn get_timestamp(&self) -> i64 {
        match self {
            CacheValue::Kline(value) => value.get_timestamp(),
            CacheValue::SMA(value) => value.get_timestamp(),
            CacheValue::BBands(value) => value.get_timestamp(),
        }
    }
}
























// 缓存管理器



