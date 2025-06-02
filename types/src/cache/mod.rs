pub mod cache_entry;
pub mod cache_key;

use crate::market::Exchange;
use serde::{Deserialize, Serialize};

use crate::market::KlineInterval;
use std::hash::Hash;
use std::fmt::Debug;
use crate::market::Kline;
use cache_key::{KlineCacheKey, IndicatorCacheKey, BacktestKlineCacheKey, BacktestIndicatorCacheKey};
use std::time::Duration;
use cache_entry::{KlineCacheEntry, IndicatorCacheEntry, HistoryKlineCacheEntry, HistoryIndicatorCacheEntry};
use crate::indicator::Indicator;
use deepsize::DeepSizeOf;
use std::sync::Arc;
use std::str::FromStr;

pub trait CacheKeyTrait{
    fn get_key(&self) -> String;
    fn get_exchange(&self) -> Exchange;
    fn get_symbol(&self) -> String;
    fn get_interval(&self) -> KlineInterval;
}



#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(tag = "key_type", content = "key_config")]
#[serde(rename_all = "lowercase")]
pub enum CacheKey {
    Kline(KlineCacheKey), // 实时K线缓存键
    Indicator(IndicatorCacheKey), // 实时指标缓存键
    BacktestKline(BacktestKlineCacheKey), // 回测K线缓存键
    BacktestIndicator(BacktestIndicatorCacheKey), // 回测指标缓存键
}

impl FromStr for CacheKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        let key_type = parts[0];
        match key_type {
            "kline" => Ok(CacheKey::Kline(KlineCacheKey::from_str(s)?)),
            "indicator" => Ok(CacheKey::Indicator(IndicatorCacheKey::from_str(s)?)),
            "backtest_kline" => Ok(CacheKey::BacktestKline(BacktestKlineCacheKey::from_str(s)?)),
            "backtest_indicator" => Ok(CacheKey::BacktestIndicator(BacktestIndicatorCacheKey::from_str(s)?)),
            _ => Err("Invalid cache key type".to_string()),
        }
    }
}

impl CacheKey {
    pub fn get_key(&self) -> String {
        match self {
            CacheKey::Kline(key) => key.get_key(),
            CacheKey::Indicator(key) => key.get_key(),
            CacheKey::BacktestKline(key) => key.get_key(),
            CacheKey::BacktestIndicator(key) => key.get_key(),
        }
    }

    pub fn get_exchange(&self) -> Exchange {
        match self {
            CacheKey::Kline(key) => key.exchange.clone(),
            CacheKey::Indicator(key) => key.exchange.clone(),
            CacheKey::BacktestKline(key) => key.exchange.clone(),
            CacheKey::BacktestIndicator(key) => key.exchange.clone(),
        }
    }

    pub fn get_symbol(&self) -> String {
        match self {
            CacheKey::Kline(key) => key.symbol.clone(),
            CacheKey::Indicator(key) => key.symbol.clone(),
            CacheKey::BacktestKline(key) => key.symbol.clone(),
            CacheKey::BacktestIndicator(key) => key.symbol.clone(),
        }
    }

    pub fn get_interval(&self) -> KlineInterval {
        match self {
            CacheKey::Kline(key) => key.interval.clone(),
            CacheKey::Indicator(key) => key.interval.clone(),
            CacheKey::BacktestKline(key) => key.interval.clone(),
            CacheKey::BacktestIndicator(key) => key.interval.clone(),
        }
    }
    
}








pub trait CacheItem: Clone + Debug + DeepSizeOf {
    fn to_json(&self) -> serde_json::Value;
    fn to_json_with_time(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn get_timestamp(&self) -> i64;
}

#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]

pub enum CacheValue {
    Kline(Kline),
    Indicator(Indicator),
}

impl CacheValue {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            CacheValue::Kline(value) => value.to_json(),
            CacheValue::Indicator(value) => value.to_json(),
        }
    }

    pub fn to_json_with_time(&self) -> serde_json::Value {
        match self {
            CacheValue::Kline(value) => value.to_json_with_time(),
            CacheValue::Indicator(value) => value.to_json_with_time(),
        }
    }

    pub fn to_list(&self) -> Vec<f64> {
        match self {
            CacheValue::Kline(value) => value.to_list(),
            CacheValue::Indicator(value) => value.to_list(),
        }
    }

    

    pub fn get_timestamp(&self) -> i64 {
        match self {
            CacheValue::Kline(value) => value.get_timestamp(),
            CacheValue::Indicator(value) => value.get_timestamp(),
        }
    }
    
    pub fn as_kline_ref(&self) -> Option<&Kline> {
        match self {
            CacheValue::Kline(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_kline(&self) -> Option<Kline> {
        match self {
            CacheValue::Kline(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn as_indicator_ref(&self) -> Option<&Indicator> {
        match self {
            CacheValue::Indicator(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_indicator(&self) -> Option<Indicator> {
        match self {
            CacheValue::Indicator(value) => Some(value.clone()),
            _ => None,
        }
    }
}



pub trait CacheEntryTrait {
    fn get_key(&self) -> CacheKey;
    fn initialize(&mut self, data: Vec<CacheValue>);
    fn update(&mut self, cache_value: CacheValue);
    fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>>;
    fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>>;
    fn get_create_time(&self) -> i64;
    fn get_update_time(&self) -> i64;
    fn get_max_size(&self) -> u32;
    fn get_is_fresh(&self) -> bool;
    fn get_ttl(&self) -> Duration;
    fn get_length(&self) -> u32;
    fn get_memory_size(&self) -> u32;
}


#[derive(Debug, Clone)]
pub enum CacheEntry {
    Kline(KlineCacheEntry),
    Indicator(IndicatorCacheEntry),
    HistoryKline(HistoryKlineCacheEntry),
    HistoryIndicator(HistoryIndicatorCacheEntry),
}

impl CacheEntry {
    pub fn new(key: CacheKey, max_size: u32, ttl: Duration) -> Self {
        match key {
            CacheKey::Kline(key) => CacheEntry::Kline(KlineCacheEntry::new(key, max_size, ttl)),
            CacheKey::Indicator(key) => CacheEntry::Indicator(IndicatorCacheEntry::new(key, max_size, ttl)),
            CacheKey::BacktestKline(key) => CacheEntry::HistoryKline(HistoryKlineCacheEntry::new(key, Some(max_size), ttl)),
            CacheKey::BacktestIndicator(key) => CacheEntry::HistoryIndicator(HistoryIndicatorCacheEntry::new(key, Some(max_size), ttl)),
        }
    }

    pub fn initialize(&mut self, data: Vec<CacheValue>) {
        match self {
            CacheEntry::Kline(entry) => entry.initialize(data),
            CacheEntry::Indicator(entry) => entry.initialize(data),
            CacheEntry::HistoryKline(entry) => entry.initialize(data),
            CacheEntry::HistoryIndicator(entry) => entry.initialize(data),
        }
    }

    pub fn update(&mut self, cache_value: CacheValue) {
        match self {
            CacheEntry::Kline(entry) => entry.update(cache_value),
            CacheEntry::Indicator(entry) => entry.update(cache_value),
            CacheEntry::HistoryKline(entry) => entry.update(cache_value),
            CacheEntry::HistoryIndicator(entry) => entry.update(cache_value),
        }
    }

    pub fn get_key(&self) -> CacheKey {
        match self {
            CacheEntry::Kline(entry) => entry.get_key(),
            CacheEntry::Indicator(entry) => entry.get_key(),
            CacheEntry::HistoryKline(entry) => entry.get_key(),
            CacheEntry::HistoryIndicator(entry) => entry.get_key(),
        }
    }
    pub fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
        match self {
            CacheEntry::Kline(entry) => entry.get_all_cache_data(),
            CacheEntry::Indicator(entry) => entry.get_all_cache_data(),
            CacheEntry::HistoryKline(entry) => entry.get_all_cache_data(),
            CacheEntry::HistoryIndicator(entry) => entry.get_all_cache_data(),
        }
    }

    pub fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        match self {
            CacheEntry::Kline(entry) => entry.get_cache_data(index, limit),
            CacheEntry::Indicator(entry) => entry.get_cache_data(index, limit),
            CacheEntry::HistoryKline(entry) => entry.get_cache_data(index, limit),
            CacheEntry::HistoryIndicator(entry) => entry.get_cache_data(index, limit),
        }
    }

    pub fn get_create_time(&self) -> i64 {
        match self {
            CacheEntry::Kline(entry) => entry.get_create_time(),
            CacheEntry::Indicator(entry) => entry.get_create_time(),
            CacheEntry::HistoryKline(entry) => entry.get_create_time(),
            CacheEntry::HistoryIndicator(entry) => entry.get_create_time(),
        }
    }

    pub fn get_update_time(&self) -> i64 {
        match self {
            CacheEntry::Kline(entry) => entry.get_update_time(),
            CacheEntry::Indicator(entry) => entry.get_update_time(),
            CacheEntry::HistoryKline(entry) => entry.get_update_time(),
            CacheEntry::HistoryIndicator(entry) => entry.get_update_time(),
        }
    }

    pub fn get_max_size(&self) -> u32 {
        match self {
            CacheEntry::Kline(entry) => entry.get_max_size(),
            CacheEntry::Indicator(entry) => entry.get_max_size(),
            CacheEntry::HistoryKline(entry) => entry.get_max_size(),
            CacheEntry::HistoryIndicator(entry) => entry.get_max_size(),
        }
    }

    pub fn get_is_fresh(&self) -> bool {
        match self {
            CacheEntry::Kline(entry) => entry.get_is_fresh(),
            CacheEntry::Indicator(entry) => entry.get_is_fresh(),
            CacheEntry::HistoryKline(entry) => entry.get_is_fresh(),
            CacheEntry::HistoryIndicator(entry) => entry.get_is_fresh(),
        }
    }

    pub fn get_ttl(&self) -> Duration {
        match self {
            CacheEntry::Kline(entry) => entry.get_ttl(),
            CacheEntry::Indicator(entry) => entry.get_ttl(),
            CacheEntry::HistoryKline(entry) => entry.get_ttl(),
            CacheEntry::HistoryIndicator(entry) => entry.get_ttl(),
        }
    }

    pub fn as_kline_cache_entry_ref(&self) -> Option<&KlineCacheEntry> {
        match self {
            CacheEntry::Kline(entry) => Some(entry),
            _ => None,
        }
    }

    pub fn as_kline_cache_entry_mut(&mut self) -> Option<&mut KlineCacheEntry> {
        match self {
            CacheEntry::Kline(entry) => Some(entry),
            _ => None,
        }
    }

    pub fn as_indicator_cache_entry_ref(&self) -> Option<&IndicatorCacheEntry> {
        match self {
            CacheEntry::Indicator(entry) => Some(entry),
            _ => None,
        }
    }

    pub fn as_indicator_cache_entry_mut(&mut self) -> Option<&mut IndicatorCacheEntry> {
        match self {
            CacheEntry::Indicator(entry) => Some(entry),
            _ => None,
        }
    }

    pub fn get_length(&self) -> u32 {
        match self {
            CacheEntry::Kline(entry) => entry.get_length(),
            CacheEntry::Indicator(entry) => entry.get_length(),
            CacheEntry::HistoryKline(entry) => entry.get_length(),
            CacheEntry::HistoryIndicator(entry) => entry.get_length(),
        }
    }

    pub fn get_memory_size(&self) -> u32 {
        match self {
            CacheEntry::Kline(entry) => entry.get_memory_size(),
            CacheEntry::Indicator(entry) => entry.get_memory_size(),
            CacheEntry::HistoryKline(entry) => entry.get_memory_size(),
            CacheEntry::HistoryIndicator(entry) => entry.get_memory_size(),
        }
    }
    


    
    
    
    
    
    
}



















// 缓存管理器



