use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use crate::indicator_config::SMAConfig;
use strum::{EnumString, Display};


#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display, Serialize, Deserialize)]
pub enum Indicators {
    // 简单移动平均线
    #[strum(serialize = "sma")]
    SimpleMovingAverage(SMAConfig),
}


pub trait IndicatorData: Debug + Send + Sync {
    fn get_data(&self) -> Vec<f64>; // 示例方法
    // fn exchange(&self) -> &Exchange;
    // fn symbol(&self) -> &String;
    // fn to_json(&self) -> serde_json::Value;
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMA {
    pub timestamp: i64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMABuffer {
    pub exchange: Exchange,
    pub symbol: String,
    pub kline_interval: KlineInterval,
    pub sma_config: SMAConfig,
    pub buffer: Vec<SMA>,
}

// impl IndicatorData for SMABuffer {
//     // fn exchange(&self) -> &Exchange {
//     //     &self.exchange
//     // }
//     // fn symbol(&self) -> &String {
//     //     &self.symbol
//     // }
//     // fn to_json(&self) -> serde_json::Value {
//     //     serde_json::to_value(self).unwrap()
//     // }
// }

