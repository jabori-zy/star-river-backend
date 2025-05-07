pub mod sma; // 简单移动平均线
pub mod bbands; // 布林带

use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use crate::indicator_config::SMAConfig;
use strum::{EnumString, Display};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum IndicatorConfig {
    // 简单移动平均线
    #[strum(serialize = "sma")]
    #[serde(rename = "sma")]
    SimpleMovingAverage(SMAConfig),
}

impl IndicatorConfig {
    pub fn update_config(&mut self, config: &Value) {
        match self {
            IndicatorConfig::SimpleMovingAverage(sma_config) => {
                if let Some(period) = config.get("period").and_then(|v| v.as_i64()) {
                    sma_config.period = period as i32;
                }
            }
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

