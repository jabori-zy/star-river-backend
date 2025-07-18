use serde::{Deserialize, Serialize};
use crate::indicator::{PriceSource, IndicatorConfigTrait, Indicator};
use std::str::FromStr;
use serde_json::{Value, json};
use deepsize::DeepSizeOf;
use crate::cache::CacheItem;
use utils::timestamp_to_utc8;
use crate::indicator::IndicatorTrait;


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MACDConfig {
    pub fast_period: i32,
    pub slow_period: i32,
    pub signal_period: i32,
    pub price_source: PriceSource,
}

impl ToString for MACDConfig {
    fn to_string(&self) -> String {
        format!("macd(fast={} slow={} signal={} source={})", self.fast_period, self.slow_period, self.signal_period, self.price_source)
    }
}

impl FromStr for MACDConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::indicator::utils::*;
        
        let (_name, params) = parse_indicator_config_from_str(s)?;
        
        let fast_period = get_required_i32_param(&params, "fast")?;
        let slow_period = get_required_i32_param(&params, "slow")?;
        let signal_period = get_required_i32_param(&params, "signal")?;
        let price_source = get_required_parsed_param::<PriceSource>(&params, "source")?;
        
        // 参数验证
        if fast_period <= 0 {
            return Err("fast参数必须大于0".to_string());
        }
        if slow_period <= 0 {
            return Err("slow参数必须大于0".to_string());
        }
        if signal_period <= 0 {
            return Err("signal参数必须大于0".to_string());
        }
        
        Ok(Self { fast_period, slow_period, signal_period, price_source })
    }
}


impl IndicatorConfigTrait for MACDConfig {
    fn new(config: &Value) -> Result<Self, String> {
        let fast_period = config.get("fastPeriod").and_then(|v| v.as_i64()).ok_or("MACD配置格式错误: fast_period 不存在".to_string())?;
        let slow_period = config.get("slowPeriod").and_then(|v| v.as_i64()).ok_or("MACD配置格式错误: slow_period 不存在".to_string())?;
        let signal_period = config.get("signalPeriod").and_then(|v| v.as_i64()).ok_or("MACD配置格式错误: signal_period 不存在".to_string())?;
        let price_source = config.get("priceSource").and_then(|v| v.as_str()).ok_or("MACD配置格式错误: priceSource 不存在".to_string())?.parse::<PriceSource>().map_err(|e| e.to_string())?;
        Ok(Self { fast_period: fast_period as i32, slow_period: slow_period as i32, signal_period: signal_period as i32, price_source })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub struct MACD {
    pub timestamp: i64,
    pub macd: f64, // 差离值
    pub signal: f64, // 讯号线
    pub histogram: f64, // 直方图
}

impl From<MACD> for Indicator {
    fn from(macd: MACD) -> Self {
        Indicator::MACD(macd)
    }
}


impl IndicatorTrait for MACD {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.macd, self.signal, self.histogram]
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": timestamp_to_utc8(self.timestamp),
                "macd": self.macd,
                "signal": self.signal,
                "histogram": self.histogram
            }
        )
    }
}


impl CacheItem for MACD {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.macd, self.signal, self.histogram]
    }
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": timestamp_to_utc8(self.timestamp),
                "macd": self.macd,
                "signal": self.signal,
                "histogram": self.histogram
            }
        )
    }
}




