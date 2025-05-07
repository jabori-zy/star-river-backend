// 布林带 Bollinger Bands

use serde::{Deserialize, Serialize};
use crate::new_cache::CacheValueTrait;
use crate::market::{Exchange, KlineInterval};


pub struct BBandsSeries {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub series: Vec<BBands>,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBands {
    pub timestamp: i64,
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

impl CacheValueTrait for BBands {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.upper, self.middle, self.lower]
    }
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}