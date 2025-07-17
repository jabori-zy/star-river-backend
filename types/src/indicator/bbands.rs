// 布林带 Bollinger Bands

use serde::{Deserialize, Serialize};
use crate::cache::CacheItem;
use deepsize::DeepSizeOf;
use serde_json::json;
use crate::indicator::{PriceSource, IndicatorConfigTrait, Indicator, IndicatorTrait, MAType};
use std::str::FromStr;
use serde_json::Value;
use ordered_float::OrderedFloat;

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
        let parts: Vec<&str> = s.split('(').collect();
        if parts.len() != 2 {
            return Err("BBands配置格式无效".to_string());
        }

        let content = parts[1].split(')').next().unwrap_or_default();
        tracing::debug!("BBandsConfig content: {}", content);
        
        let param_parts: Vec<&str> = content.split_whitespace().collect();
        tracing::debug!("BBandsConfig param_parts: {:?}", param_parts);

        if param_parts.len() != 3 {
            return Err("BBands参数数量无效，应为3个参数".to_string());
        }

        let mut period = 0i32;
        let mut dev_up = OrderedFloat::from(0.0f64);
        let mut dev_down = OrderedFloat::from(0.0f64);
        let mut price_source: Option<PriceSource> = None;
        let mut ma_type: Option<MAType> = None;

        for param in param_parts {
            let kv: Vec<&str> = param.split('=').collect();
            if kv.len() != 2 {
                return Err(format!("BBands参数格式无效: {}", param));
            }
            
            let key = kv[0].trim();
            let value = kv[1].trim();

            match key {
                "period" => period = value.parse::<i32>().map_err(|e| format!("period参数解析失败: {}", e))?,
                "dev_up" => dev_up = OrderedFloat::from(value.parse::<f64>().map_err(|e| format!("dev_up参数解析失败: {}", e))?),
                "dev_down" => dev_down = OrderedFloat::from(value.parse::<f64>().map_err(|e| format!("dev_down参数解析失败: {}", e))?),
                "ma_type" => ma_type = Some(value.parse::<MAType>().map_err(|e| format!("matype参数解析失败: {}", e))?),
                "source" => price_source = Some(value.parse::<PriceSource>().map_err(|e| format!("source参数解析失败: {}", e))?),
                _ => return Err(format!("BBands参数格式无效: {}", param)),
            }
        }

        let price_source = price_source.ok_or("缺少source参数".to_string())?;

        Ok(BBandsConfig {
            period,
            dev_up,
            dev_down,
            price_source,
            ma_type: ma_type.ok_or("缺少ma_type参数".to_string())?,
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
