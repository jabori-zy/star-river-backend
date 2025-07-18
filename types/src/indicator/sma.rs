use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::cache::CacheItem;
use crate::indicator::Indicator;
use serde_json::Value;
use crate::indicator::IndicatorConfigTrait;
use deepsize::DeepSizeOf;
use std::str::FromStr;
use serde_json::json;
use crate::indicator::IndicatorTrait;


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SMAConfig {
    pub period: i32,
}

impl ToString for SMAConfig {
    fn to_string(&self) -> String {
        format!("sma(period={})", self.period)
    }
}

impl FromStr for SMAConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::indicator::utils::*;
        
        let (_name, params) = parse_indicator_config_from_str(s)?;
        let period = get_required_i32_param(&params, "period")?;
        
        Ok(SMAConfig { period })
    }
}

impl IndicatorConfigTrait for SMAConfig {
    fn new(config: &Value) -> Result<Self, String> {
        if let Some(period) = config.get("period").and_then(|v| v.as_i64()) {
            Ok(Self { period: period as i32 })
        } else {
            Err("SMA配置格式错误".to_string())
        }
    }
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


impl IndicatorTrait for SMA {
    fn to_json(&self) -> Value {
        json!(
            {
                "timestamp": self.timestamp,
                "sma": self.sma
            }
        )
        
    }

    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.sma]
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "sma": self.sma
            }
        )
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
    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "sma": self.sma
            }
        )
    }
}
