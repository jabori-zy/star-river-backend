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
        let parts: Vec<&str> = s.split('(').collect();
        if parts.len() != 2 {
            return Err("SMA配置格式无效".to_string());
        }

        // 移除右括号并获取内容
        let content = parts[1].split(')').next().unwrap_or_default();
        
        // 只支持 "period=9" 格式
        if content.contains("=") {
            let kv_parts: Vec<&str> = content.split('=').collect();
            if kv_parts.len() != 2 || kv_parts[0].trim() != "period" {
                return Err("SMA参数格式无效，应为 'period=值'".to_string());
            }
            
            let period = kv_parts[1].trim().parse::<i32>().map_err(|e| e.to_string())?;
            Ok(SMAConfig { period })
        } else {
            return Err("SMA配置格式无效，应为 'sma(period=值)'".to_string());
        }
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
