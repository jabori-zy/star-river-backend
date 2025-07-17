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
        let parts: Vec<&str> = s.split('(').collect();
        tracing::debug!("MACDConfig parts: {:?}", parts);
        if parts.len() != 2 {
            return Err("MACD配置格式无效".to_string());
        }

        let content = parts[1].split(')').next().unwrap_or_default();
        tracing::debug!("MACDConfig content: {}", content);
        
        // 按空格分割参数
        let param_parts: Vec<&str> = content.split_whitespace().collect();
        tracing::debug!("MACDConfig param_parts: {:?}", param_parts);
        
        if param_parts.len() != 4 {
            return Err("MACD参数数量无效，应为4个参数".to_string());
        }
        
        let mut fast_period = 0i32;
        let mut slow_period = 0i32;
        let mut signal_period = 0i32;
        let mut price_source: Option<PriceSource> = None;
        
        // 解析每个键值对
        for param in param_parts {
            let kv: Vec<&str> = param.split('=').collect();
            if kv.len() != 2 {
                return Err(format!("MACD参数格式无效: {}", param));
            }
            
            let key = kv[0].trim();
            let value = kv[1].trim();
            
            match key {
                "fast" => {
                    fast_period = value.parse::<i32>().map_err(|e| format!("fast参数解析失败: {}", e))?;
                    if fast_period <= 0 {
                        return Err("fast参数必须大于0".to_string());
                    }
                },
                "slow" => {
                    slow_period = value.parse::<i32>().map_err(|e| format!("slow参数解析失败: {}", e))?;
                    if slow_period <= 0 {
                        return Err("slow参数必须大于0".to_string());
                    }
                },
                "signal" => {
                    signal_period = value.parse::<i32>().map_err(|e| format!("signal参数解析失败: {}", e))?;
                    if signal_period <= 0 {
                        return Err("signal参数必须大于0".to_string());
                    }
                },
                "source" => {
                    price_source = Some(value.parse::<PriceSource>().map_err(|e| format!("source参数解析失败: {}", e))?);
                },
                _ => return Err(format!("未知的MACD参数: {}", key)),
            }
        }
        
        let price_source = price_source.ok_or("缺少source参数".to_string())?;
        
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




