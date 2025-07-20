use crate::indicator::{PriceSource, IndicatorConfigTrait, IndicatorTrait};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use serde_json::{Value, json};
use crate::cache::CacheItem;
use deepsize::DeepSizeOf;
use utils::timestamp_to_utc8;
use crate::indicator::talib_types::IndicatorParam;



#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RSIConfig {
    pub period: i32,
    pub price_source: PriceSource,
}

impl ToString for RSIConfig {
    fn to_string(&self) -> String {
        format!("rsi(period={} source={})", self.period, self.price_source)
    }
}


impl FromStr for RSIConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::indicator::utils::*;

        let (_name, params) = parse_indicator_config_from_str(s)?;
        let period = get_required_i32_param(&params, "period")?;
        let price_source = get_required_special_param::<PriceSource>(&params, "source")?;
        Ok(RSIConfig { period, price_source })
    }
}

impl IndicatorConfigTrait for RSIConfig {
    fn new(config: &Value) -> Result<Self, String> {
        let period = config.get("period")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or("RSI配置缺少period参数")?;

        let price_source = config.get("price_source")
            .and_then(|v| v.as_str())
            .and_then(|s| PriceSource::from_str(s).ok())
            .unwrap_or(PriceSource::Close);

        Ok(RSIConfig { period, price_source })
    }

    fn to_tablib_params(&self) -> Vec<IndicatorParam> {
        vec![
            IndicatorParam::TimePeriod(self.period), 
            IndicatorParam::PriceSource(self.price_source.clone())
        ]
    }
}

// RSI指标数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, DeepSizeOf)]
pub struct RSI {
    pub timestamp: i64,
    pub value: f64,
}

impl CacheItem for RSI {
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    fn to_json(&self) -> Value {
        json!({
            "timestamp": self.timestamp,
            "value": self.value
        })
    }

    fn to_json_with_time(&self) -> Value {
        json!({
            "timestamp": self.timestamp,
            "time": timestamp_to_utc8(self.timestamp),
            "value": self.value
        })
    }

    fn to_list(&self) -> Vec<f64> {
        vec![self.value]
    }
}

impl IndicatorTrait for RSI {
    fn to_json(&self) -> Value {
        CacheItem::to_json(self)
    }

    fn to_list(&self) -> Vec<f64> {
        vec![self.value]
    }

    fn to_json_with_time(&self) -> Value {
        CacheItem::to_json_with_time(self)
    }
}

