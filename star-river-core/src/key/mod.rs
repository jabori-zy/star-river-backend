// pub mod cache_entry;
pub mod key;

use crate::market::Exchange;
use serde::{Deserialize, Serialize};

use crate::error::star_river_error::*;
use crate::market::KlineInterval;
use crate::strategy::TimeRange;
// use cache_entry::{IndicatorCacheEntry, KlineCacheEntry};
use key::{IndicatorKey, KlineKey};
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

pub trait KeyTrait: Clone + Debug {
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