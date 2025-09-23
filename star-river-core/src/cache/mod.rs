pub mod cache_entry;
pub mod key;

use crate::market::Exchange;
use serde::{Deserialize, Serialize};

use crate::error::star_river_error::*;
use crate::indicator::Indicator;
use crate::market::Kline;
use crate::market::KlineInterval;
use crate::strategy::TimeRange;
use cache_entry::{IndicatorCacheEntry, KlineCacheEntry};
use chrono::{DateTime, Utc};
use deepsize::DeepSizeOf;
use key::{IndicatorKey, KlineKey};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

pub trait KeyTrait {
    fn get_key_str(&self) -> String;
    fn get_exchange(&self) -> Exchange;
    fn get_symbol(&self) -> String;
    fn get_interval(&self) -> KlineInterval;
    fn get_time_range(&self) -> Option<TimeRange>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(tag = "key_type", content = "key_config")]
#[serde(rename_all = "lowercase")]
pub enum Key {
    // Kline(KlineKey), // 实时K线缓存键
    // Indicator(IndicatorKey), // 实时指标缓存键
    Kline(KlineKey),         // 回测K线缓存键
    Indicator(IndicatorKey), // 回测指标缓存键
}

impl FromStr for Key {
    type Err = StarRiverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        let key_type = parts[0];
        match key_type {
            // "kline" => Ok(Key::Kline(KlineKey::from_str(s)?)),
            // "indicator" => Ok(Key::Indicator(IndicatorKey::from_str(s)?)),
            "kline" => Ok(Key::Kline(KlineKey::from_str(s)?)),
            "indicator" => Ok(Key::Indicator(IndicatorKey::from_str(s)?)),
            _ => Err(InvalidKeyTypeSnafu {
                key_type: key_type.to_string(),
            }
            .build()),
        }
    }
}

impl Key {
    pub fn get_key_str(&self) -> String {
        match self {
            // Key::Kline(key) => key.get_key_str(),
            // Key::Indicator(key) => key.get_key_str(),
            Key::Kline(key) => key.get_key_str(),
            Key::Indicator(key) => key.get_key_str(),
        }
    }

    pub fn get_exchange(&self) -> Exchange {
        match self {
            // Key::Kline(key) => key.exchange.clone(),
            // Key::Indicator(key) => key.exchange.clone(),
            Key::Kline(key) => key.exchange.clone(),
            Key::Indicator(key) => key.exchange.clone(),
        }
    }

    pub fn get_symbol(&self) -> String {
        match self {
            // Key::Kline(key) => key.symbol.clone(),
            // Key::Indicator(key) => key.symbol.clone(),
            Key::Kline(key) => key.symbol.clone(),
            Key::Indicator(key) => key.symbol.clone(),
        }
    }

    pub fn get_interval(&self) -> KlineInterval {
        match self {
            // Key::Kline(key) => key.interval.clone(),
            // Key::Indicator(key) => key.interval.clone(),
            Key::Kline(key) => key.interval.clone(),
            Key::Indicator(key) => key.interval.clone(),
        }
    }

    pub fn get_start_time(&self) -> Option<String> {
        match self {
            Key::Kline(key) => key.start_time.clone(),
            Key::Indicator(key) => key.start_time.clone(),
        }
    }

    pub fn get_end_time(&self) -> Option<String> {
        match self {
            Key::Kline(key) => key.end_time.clone(),
            Key::Indicator(key) => key.end_time.clone(),
        }
    }

    pub fn get_time_range(&self) -> Option<TimeRange> {
        match self {
            Key::Kline(key) => {
                if let (Some(start_time), Some(end_time)) = (key.start_time.clone(), key.end_time.clone()) {
                    Some(TimeRange::new(start_time, end_time))
                } else {
                    None
                }
            }
            Key::Indicator(key) => {
                if let (Some(start_time), Some(end_time)) = (key.start_time.clone(), key.end_time.clone()) {
                    Some(TimeRange::new(start_time, end_time))
                } else {
                    None
                }
            }
        }
    }
}

pub trait CacheItem: Clone + Debug + DeepSizeOf {
    fn to_json(&self) -> serde_json::Value;
    fn to_json_with_time(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn get_timestamp(&self) -> i64;
    fn get_datetime(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]

pub enum CacheValue {
    Kline(Kline),
    Indicator(Indicator),
}

impl CacheItem for CacheValue {
    fn to_json(&self) -> serde_json::Value {
        match self {
            CacheValue::Kline(value) => value.to_json(),
            CacheValue::Indicator(value) => value.to_json(),
        }
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        match self {
            CacheValue::Kline(value) => value.to_json_with_time(),
            CacheValue::Indicator(value) => value.to_json_with_time(),
        }
    }

    fn to_list(&self) -> Vec<f64> {
        match self {
            CacheValue::Kline(value) => value.to_list(),
            CacheValue::Indicator(value) => value.to_list(),
        }
    }

    fn get_datetime(&self) -> DateTime<Utc> {
        match self {
            CacheValue::Kline(value) => value.get_datetime(),
            CacheValue::Indicator(value) => value.get_datetime(),
        }
    }

    fn get_timestamp(&self) -> i64 {
        match self {
            CacheValue::Kline(value) => value.get_timestamp(),
            CacheValue::Indicator(value) => value.get_timestamp(),
        }
    }
}

impl CacheValue {
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
    fn get_key(&self) -> Key;
    fn initialize(&mut self, data: Vec<CacheValue>);
    fn update(&mut self, cache_value: CacheValue);
    fn clear(&mut self);
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
}

impl CacheEntry {
    pub fn new(key: Key, max_size: u32, ttl: Duration) -> Self {
        match key {
            // Key::Kline(key) => CacheEntry::Kline(KlineCacheEntry::new(key, max_size, ttl)),
            // Key::Indicator(key) => CacheEntry::Indicator(IndicatorCacheEntry::new(key, max_size, ttl)),
            Key::Kline(key) => CacheEntry::Kline(KlineCacheEntry::new(key, Some(max_size), ttl)),
            Key::Indicator(key) => CacheEntry::Indicator(IndicatorCacheEntry::new(key, Some(max_size), ttl)),
        }
    }

    pub fn initialize(&mut self, data: Vec<CacheValue>) {
        match self {
            // CacheEntry::Kline(entry) => entry.initialize(data),
            // CacheEntry::Indicator(entry) => entry.initialize(data),
            CacheEntry::Kline(entry) => entry.initialize(data),
            CacheEntry::Indicator(entry) => entry.initialize(data),
        }
    }

    pub fn update(&mut self, cache_value: CacheValue) {
        match self {
            // CacheEntry::Kline(entry) => entry.update(cache_value),
            // CacheEntry::Indicator(entry) => entry.update(cache_value),
            CacheEntry::Kline(entry) => entry.update(cache_value),
            CacheEntry::Indicator(entry) => entry.update(cache_value),
        }
    }

    pub fn clear(&mut self) {
        match self {
            CacheEntry::Kline(entry) => entry.clear(),
            CacheEntry::Indicator(entry) => entry.clear(),
        }
    }

    pub fn get_key(&self) -> Key {
        match self {
            // CacheEntry::Kline(entry) => entry.get_key(),
            // CacheEntry::Indicator(entry) => entry.get_key(),
            CacheEntry::Kline(entry) => entry.get_key(),
            CacheEntry::Indicator(entry) => entry.get_key(),
        }
    }
    pub fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
        match self {
            // CacheEntry::Kline(entry) => entry.get_all_cache_data(),
            // CacheEntry::Indicator(entry) => entry.get_all_cache_data(),
            CacheEntry::Kline(entry) => entry.get_all_cache_data(),
            CacheEntry::Indicator(entry) => entry.get_all_cache_data(),
        }
    }

    pub fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        match self {
            // CacheEntry::Kline(entry) => entry.get_cache_data(index, limit),
            // CacheEntry::Indicator(entry) => entry.get_cache_data(index, limit),
            CacheEntry::Kline(entry) => entry.get_cache_data(index, limit),
            CacheEntry::Indicator(entry) => entry.get_cache_data(index, limit),
        }
    }

    pub fn get_create_time(&self) -> i64 {
        match self {
            // CacheEntry::Kline(entry) => entry.get_create_time(),
            // CacheEntry::Indicator(entry) => entry.get_create_time(),
            CacheEntry::Kline(entry) => entry.get_create_time(),
            CacheEntry::Indicator(entry) => entry.get_create_time(),
        }
    }

    pub fn get_update_time(&self) -> i64 {
        match self {
            // CacheEntry::Kline(entry) => entry.get_update_time(),
            // CacheEntry::Indicator(entry) => entry.get_update_time(),
            CacheEntry::Kline(entry) => entry.get_update_time(),
            CacheEntry::Indicator(entry) => entry.get_update_time(),
        }
    }

    pub fn get_max_size(&self) -> u32 {
        match self {
            // CacheEntry::Kline(entry) => entry.get_max_size(),
            // CacheEntry::Indicator(entry) => entry.get_max_size(),
            CacheEntry::Kline(entry) => entry.get_max_size(),
            CacheEntry::Indicator(entry) => entry.get_max_size(),
        }
    }

    pub fn get_is_fresh(&self) -> bool {
        match self {
            // CacheEntry::Kline(entry) => entry.get_is_fresh(),
            // CacheEntry::Indicator(entry) => entry.get_is_fresh(),
            CacheEntry::Kline(entry) => entry.get_is_fresh(),
            CacheEntry::Indicator(entry) => entry.get_is_fresh(),
        }
    }

    pub fn get_ttl(&self) -> Duration {
        match self {
            // CacheEntry::Kline(entry) => entry.get_ttl(),
            // CacheEntry::Indicator(entry) => entry.get_ttl(),
            CacheEntry::Kline(entry) => entry.get_ttl(),
            CacheEntry::Indicator(entry) => entry.get_ttl(),
        }
    }

    // pub fn as_kline_cache_entry_ref(&self) -> Option<&KlineCacheEntry> {
    //     match self {
    //         CacheEntry::Kline(entry) => Some(entry),
    //         _ => None,
    //     }
    // }
    //
    // pub fn as_kline_cache_entry_mut(&mut self) -> Option<&mut KlineCacheEntry> {
    //     match self {
    //         CacheEntry::Kline(entry) => Some(entry),
    //         _ => None,
    //     }
    // }
    //
    // pub fn as_indicator_cache_entry_ref(&self) -> Option<&IndicatorCacheEntry> {
    //     match self {
    //         CacheEntry::Indicator(entry) => Some(entry),
    //         _ => None,
    //     }
    // }
    //
    // pub fn as_indicator_cache_entry_mut(&mut self) -> Option<&mut IndicatorCacheEntry> {
    //     match self {
    //         CacheEntry::Indicator(entry) => Some(entry),
    //         _ => None,
    //     }
    // }

    pub fn get_length(&self) -> u32 {
        match self {
            // CacheEntry::Kline(entry) => entry.get_length(),
            // CacheEntry::Indicator(entry) => entry.get_length(),
            CacheEntry::Kline(entry) => entry.get_length(),
            CacheEntry::Indicator(entry) => entry.get_length(),
        }
    }

    pub fn get_memory_size(&self) -> u32 {
        match self {
            // CacheEntry::Kline(entry) => entry.get_memory_size(),
            // CacheEntry::Indicator(entry) => entry.get_memory_size(),
            CacheEntry::Kline(entry) => entry.get_memory_size(),
            CacheEntry::Indicator(entry) => entry.get_memory_size(),
        }
    }
}

// 缓存管理器
