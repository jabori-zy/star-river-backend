pub mod sma; // 简单移动平均线
pub mod bbands; // 布林带


use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use strum::Display;
use std::collections::HashMap;
use crate::indicator::sma::SMA;
use crate::indicator::bbands::BBands;
use crate::cache::CacheItem;
use crate::indicator::sma::SMAConfig;
use deepsize::DeepSizeOf;
use crate::cache::CacheValue;

pub trait IndicatorConfigTrait {
    fn new(config: &Value) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum IndicatorConfig {
    // 简单移动平均线
    #[serde(rename = "sma")]
    SMA(SMAConfig),
}

impl ToString for IndicatorConfig {
    fn to_string(&self) -> String {
        match self {
            IndicatorConfig::SMA(sma_config) => format!("sma(period={})", sma_config.period),
        }
    }
}


impl IndicatorConfig {
    pub fn new(indicator_type: &str, config: &Value) -> Self {
        match indicator_type {
            "sma" => IndicatorConfig::SMA(SMAConfig::new(config)),
            _ => panic!("Invalid indicator type"),
        }
    }
}




#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub enum Indicator {
    SMA(SMA),
    BBands(BBands),
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
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    fn to_list(&self) -> Vec<f64> {
        match self {
            Indicator::SMA(sma) => sma.to_list(),
            Indicator::BBands(bbands) => bbands.to_list(),
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

