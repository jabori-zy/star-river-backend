use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use crate::indicator_config::SMAConfig;
use strum::{EnumString, Display};
use std::collections::HashMap;
use crate::indicator::IndicatorData;
use crate::new_cache::{CacheValue, CacheValueTrait};


pub struct SMASeries {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub series: Vec<SMA>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMA {
    pub timestamp: i64,
    pub sma: f64,
}

impl TryFrom<CacheValue> for SMA {
    type Error = String;

    fn try_from(value: CacheValue) -> Result<Self, Self::Error> {
        match value {
            CacheValue::SMA(sma) => Ok(sma),
            _ => Err(format!("无法将CacheValue转换为SMA: {:?}", value)),
        }
    }
}

impl CacheValueTrait for SMA {
    fn to_cache_value(&self) -> CacheValue {
        CacheValue::SMA(self.clone())
    }
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.sma]
    }
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}