use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::cache::CacheItem;
use crate::indicator::Indicator;
use serde_json::Value;
use crate::indicator::IndicatorConfigTrait;
use deepsize::DeepSizeOf;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SMAConfig {
    pub period: i32,
}

impl IndicatorConfigTrait for SMAConfig {
    fn new(config: &Value) -> Self {
        if let Some(period) = config.get("period").and_then(|v| v.as_i64()) {
            Self { period: period as i32 }
        } else {
            Self { period: 9 }
        }
    }
}



pub struct SMASeries {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub series: Vec<SMA>,
}



#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub struct SMA {
    pub timestamp: i64,
    pub sma: f64,
}

impl From<SMA> for Indicator {
    fn from(sma: SMA) -> Self {
        Indicator::SMA(sma)
    }
}


impl CacheItem for SMA {
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