// pub mod cache_entry;
pub mod error;
pub mod key;

use std::{fmt::Debug, hash::Hash, str::FromStr};

// use cache_entry::{IndicatorCacheEntry, KlineCacheEntry};
pub use key::{IndicatorKey, KlineKey};
use serde::{Deserialize, Serialize};
use star_river_core::{exchange::Exchange, kline::KlineInterval, system::TimeRange};

use crate::error::{InvalidKeyTypeSnafu, KeyError};

pub trait KeyTrait: Clone + Debug {
    fn key_str(&self) -> String;
    fn exchange(&self) -> Exchange;
    fn symbol(&self) -> String;
    fn interval(&self) -> KlineInterval;
    fn time_range(&self) -> Option<TimeRange>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(tag = "key_type", content = "key_config")]
#[serde(rename_all = "lowercase")]
pub enum Key {
    Kline(KlineKey),         // 回测K线缓存键
    Indicator(IndicatorKey), // 回测指标缓存键
}

impl FromStr for Key {
    type Err = KeyError;

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
            Key::Kline(key) => key.key_str(),
            Key::Indicator(key) => key.key_str(),
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
