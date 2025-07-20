// 布林带 Bollinger Bands

use serde::{Deserialize, Serialize};
use crate::cache::CacheItem;
use deepsize::DeepSizeOf;
use serde_json::json;
use crate::indicator::{PriceSource, IndicatorConfigTrait, Indicator, IndicatorTrait, MAType};
use std::str::FromStr;
use serde_json::Value;
use ordered_float::OrderedFloat;
use crate::indicator::talib_types::IndicatorParam;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BBandsConfig {
    pub period: i32,
    pub dev_up: OrderedFloat<f64>,
    pub dev_down: OrderedFloat<f64>,
    pub price_source: PriceSource,
    pub ma_type: MAType,
}

impl ToString for BBandsConfig {
    fn to_string(&self) -> String {
        format!("bbands(period={} dev_up={} dev_down={} source={} ma_type={})", self.period, self.dev_up, self.dev_down, self.price_source, self.ma_type)
    }
}

impl FromStr for BBandsConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::indicator::utils::*;
        
        let (_name, params) = parse_indicator_config_from_str(s)?;
        
        let period = get_required_i32_param(&params, "period")?;
        let dev_up = OrderedFloat::from(get_required_f64_param(&params, "dev_up")?);
        let dev_down = OrderedFloat::from(get_required_f64_param(&params, "dev_down")?);
        let price_source = get_required_special_param::<PriceSource>(&params, "source")?;
        let ma_type = get_required_special_param::<MAType>(&params, "ma_type")?;
        
        Ok(BBandsConfig {
            period,
            dev_up,
            dev_down,
            price_source,
            ma_type,
        })
    }
}

impl IndicatorConfigTrait for BBandsConfig {
    fn new(config: &Value) -> Result<Self, String> {
        let period = config.get("period").and_then(|v| v.as_i64()).ok_or("BBands配置格式错误: period 不存在".to_string())?;
        let dev_up = config.get("devUp").and_then(|v| v.as_f64()).ok_or("BBands配置格式错误: devUp 不存在".to_string())?;
        let dev_down = config.get("devDown").and_then(|v| v.as_f64()).ok_or("BBands配置格式错误: devDown 不存在".to_string())?;
        let ma_type = config.get("maType").and_then(|v| v.as_str()).ok_or("BBands配置格式错误: maType 不存在".to_string())?.parse::<MAType>().map_err(|e| e.to_string())?;
        let price_source = config.get("priceSource").and_then(|v| v.as_str()).ok_or("BBands配置格式错误: priceSource 不存在".to_string())?.parse::<PriceSource>().map_err(|e| e.to_string())?;
        Ok(Self { period: period as i32, dev_up: OrderedFloat::from(dev_up), dev_down: OrderedFloat::from(dev_down), price_source, ma_type })
    }

    fn to_tablib_params(&self) -> Vec<IndicatorParam> {
        vec![
            IndicatorParam::TimePeriod(self.period), 
            IndicatorParam::Deviation(self.dev_up.into_inner()), 
            IndicatorParam::Deviation(self.dev_down.into_inner()), 
            IndicatorParam::PriceSource(self.price_source.clone()), 
            IndicatorParam::MAType(self.ma_type.clone())
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub struct BBands {
    pub timestamp: i64,
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}
impl From<BBands> for Indicator {
    fn from(bbands: BBands) -> Self {
        Indicator::BBands(bbands)
    }
}


impl IndicatorTrait for BBands {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.upper, self.middle, self.lower]
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "upper": self.upper,
                "middle": self.middle,
                "lower": self.lower
            }
        )
    }
}

impl CacheItem for BBands {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.upper, self.middle, self.lower]
    }
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "upper": self.upper,
                "middle": self.middle,
                "lower": self.lower
            }
        )
    }
}
