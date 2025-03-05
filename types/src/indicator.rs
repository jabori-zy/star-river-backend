use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::any::Any;
use crate::indicator_config::SMAConfig;
use strum::{EnumString, Display};


#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Indicators {
    // 简单移动平均线
    #[strum(serialize = "sma")]
    #[serde(rename = "sma")]
    SimpleMovingAverage(SMAConfig),
}

#[typetag::serde(tag = "type")]
pub trait IndicatorData: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn IndicatorData>;
}

impl Clone for Box<dyn IndicatorData> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMA {
    pub timestamp: i64,
    pub value: f64,
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMASeries {
    pub exchange: Exchange,
    pub symbol: String,
    pub kline_interval: KlineInterval,
    pub sma_config: SMAConfig,
    pub data: Vec<SMA>,
}

#[typetag::serde]
impl IndicatorData for SMASeries {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn IndicatorData> {
        Box::new(self.clone())
    }
}

