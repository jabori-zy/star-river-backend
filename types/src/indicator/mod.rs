
pub mod sma; // 简单移动平均线
pub mod bbands; // 布林带
pub mod macd; // 指数平滑异同移动平均线
pub mod rsi; // 相对强弱指数
pub mod utils; // 指标配置解析工具


use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use std::collections::HashMap;
use crate::indicator::sma::SMA;
use crate::indicator::bbands::BBands;
use crate::indicator::macd::MACD;
use crate::cache::{CacheItem, CacheValue};
use crate::indicator::sma::SMAConfig;
use crate::indicator::macd::MACDConfig;
use crate::indicator::bbands::BBandsConfig;
use deepsize::DeepSizeOf;
use std::str::FromStr;
use strum::{EnumString, Display};

// 价格来源
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
pub enum PriceSource {
    #[strum(serialize = "close")]
    Close,
    #[strum(serialize = "open")]
    Open,
    #[strum(serialize = "high")]
    High,
    #[strum(serialize = "low")]
    Low,
}

#[derive(Debug, Clone, Hash, Eq,PartialEq, Serialize, Deserialize, EnumString, Display)]
pub enum MAType {
    #[strum(serialize = "sma")]
    SMA,
    #[strum(serialize = "ema")]
    EMA,
    #[strum(serialize = "wma")]
    WMA,
    #[strum(serialize = "dema")]
    DEMA,
    #[strum(serialize = "tema")]
    TEMA,
    #[strum(serialize = "trima")]
    TRIMA,
    #[strum(serialize = "kama")]
    KAMA,
    #[strum(serialize = "mama")]
    MAMA,
    #[strum(serialize = "t3")]
    T3,
}

pub trait IndicatorConfigTrait {
    fn new(config: &Value) -> Result<Self, String> where Self: Sized; // 创建指标配置,有可能失败
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "indicator_type", content = "indicator_config")]
pub enum IndicatorConfig {
    // 简单移动平均线
    #[serde(rename = "sma")]
    SMA(SMAConfig),

    #[serde(rename = "macd")]
    MACD(MACDConfig),
    
    #[serde(rename = "bbands")]
    BBands(BBandsConfig),
}

impl ToString for IndicatorConfig {
    fn to_string(&self) -> String {
        match self {
            IndicatorConfig::SMA(sma_config) => sma_config.to_string(),
            IndicatorConfig::MACD(macd_config) => macd_config.to_string(),
            IndicatorConfig::BBands(bbands_config) => bbands_config.to_string(),
        }
    }
}

impl FromStr for IndicatorConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 提取指标类型（如"sma"）
        let indicator_type = if s.contains("(") {
            s.split("(").next().unwrap_or("").trim()
        } else {
            s
        };

        // 根据指标类型创建相应的配置
        match indicator_type {
            "sma" => Ok(IndicatorConfig::SMA(SMAConfig::from_str(s)?)),
            "macd" => Ok(IndicatorConfig::MACD(MACDConfig::from_str(s)?)),
            "bbands" => Ok(IndicatorConfig::BBands(BBandsConfig::from_str(s)?)),
            _ => Err(format!("不支持的指标类型: {}", indicator_type)),
        }
    }
}

impl IndicatorConfig {
    pub fn new(indicator_type: &str, config: &Value) -> Result<Self, String> {
        match indicator_type {
            "sma" => Ok(IndicatorConfig::SMA(SMAConfig::new(config)?)),
            "macd" => Ok(IndicatorConfig::MACD(MACDConfig::new(config)?)),
            "bbands" => Ok(IndicatorConfig::BBands(BBandsConfig::new(config)?)),
            _ => Err(format!("不支持的指标类型: {}", indicator_type)),
        }
    }
}


pub trait IndicatorTrait {
    fn to_json(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn to_json_with_time(&self) -> serde_json::Value;
}



#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub enum Indicator {
    SMA(SMA),
    BBands(BBands),
    MACD(MACD),
}


impl IndicatorTrait for Indicator {
    fn to_json(&self) -> serde_json::Value {
        match self {
            Indicator::SMA(sma) => IndicatorTrait::to_json(sma),
            Indicator::BBands(bbands) => IndicatorTrait::to_json(bbands),
            Indicator::MACD(macd) => IndicatorTrait::to_json(macd),
        }
    }

    fn to_list(&self) -> Vec<f64> {
        match self {
            Indicator::SMA(sma) => IndicatorTrait::to_list(sma),
            Indicator::BBands(bbands) => IndicatorTrait::to_list(bbands),
            Indicator::MACD(macd) => IndicatorTrait::to_list(macd),
        }
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        match self {
            Indicator::SMA(sma) => IndicatorTrait::to_json_with_time(sma),
            Indicator::BBands(bbands) => IndicatorTrait::to_json_with_time(bbands),
            Indicator::MACD(macd) => IndicatorTrait::to_json_with_time(macd),
        }
    }
}


impl From<Indicator> for CacheValue {
    fn from(indicator: Indicator) -> Self {
        CacheValue::Indicator(indicator)
    }
}


impl CacheItem for Indicator {
    fn get_timestamp(&self) -> i64 {
        match self {
            Indicator::SMA(sma) => sma.timestamp,
            Indicator::BBands(bbands) => bbands.timestamp,
            Indicator::MACD(macd) => macd.timestamp,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        match self {
            Indicator::SMA(sma) => CacheItem::to_json(sma),
            Indicator::BBands(bbands) => CacheItem::to_json(bbands),
            Indicator::MACD(macd) => CacheItem::to_json(macd),
        }
    }

    fn to_list(&self) -> Vec<f64> {
        match self {
            Indicator::SMA(sma) => CacheItem::to_list(sma),
            Indicator::BBands(bbands) => CacheItem::to_list(bbands),
            Indicator::MACD(macd) => CacheItem::to_list(macd),
        }
    }

    fn to_json_with_time(&self) -> serde_json::Value {
        match self {
            Indicator::SMA(sma) => CacheItem::to_json_with_time(sma),
            Indicator::BBands(bbands) => CacheItem::to_json_with_time(bbands),
            Indicator::MACD(macd) => CacheItem::to_json_with_time(macd),
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait IndicatorData: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn IndicatorData>;
    fn get_indicator_value(&self) -> HashMap<String, Vec<IndicatorValue>>;
    fn get_latest_indicator_value(&self) -> HashMap<String, IndicatorValue>; // 获取最新指标值
}

impl Clone for Box<dyn IndicatorData> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub timestamp: i64,
    pub value: f64,
}




#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMAIndicator {
    pub exchange: Exchange,
    pub symbol: String,
    pub kline_interval: KlineInterval,
    pub indicator_config: SMAConfig,
    pub indicator_value: HashMap<String, Vec<IndicatorValue>>,
}

#[typetag::serde]
impl IndicatorData for SMAIndicator {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn IndicatorData> {
        Box::new(self.clone())
    }
    fn get_indicator_value(&self) -> HashMap<String, Vec<IndicatorValue>> {
        self.indicator_value.clone()
    }
    fn get_latest_indicator_value(&self) -> HashMap<String, IndicatorValue> {
        self.indicator_value.iter().map(|(key, value)| {
            let latest_value = value.last().unwrap();
            (key.clone(), latest_value.clone())
        }).collect()
    }
}

