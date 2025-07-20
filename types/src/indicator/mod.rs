
// pub mod sma; // 简单移动平均线
// pub mod bbands; // 布林带
// pub mod macd; // 指数平滑异同移动平均线
// pub mod rsi; // 相对强弱指数
pub mod utils; // 指标配置解析工具
// pub mod talib_types; // TA-Lib 类型定义
// pub mod registry; // 指标注册表
pub mod indicator_macros;
pub mod indicator;


use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use std::collections::HashMap;

use crate::cache::{CacheItem, CacheValue};

use crate::indicator::indicator::*;
use deepsize::DeepSizeOf;
use std::str::FromStr;
use strum::{EnumString, Display};
use crate::{impl_indicator, impl_indicator_config};

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
    SMA, // 简单移动平均线
    #[strum(serialize = "ema")]
    EMA, // Exponential Moving Average (EMA) 指数移动平均线
    #[strum(serialize = "wma")]
    WMA, // Weighted Moving Average (WMA) 加权移动平均线
    #[strum(serialize = "dema")]
    DEMA, // Double Exponential Moving Average (DEMA) 双指数移动平均线
    #[strum(serialize = "tema")]
    TEMA, // Triple Exponential Moving Average (TEMA) 三指数移动平均线
    #[strum(serialize = "trima")]
    TRIMA, // Triangular Moving Average (TRIMA) 三角形移动平均线
    #[strum(serialize = "kama")]
    KAMA, // Kaufman Adaptive Moving Average (KAMA) 卡夫曼自适应移动平均线
    #[strum(serialize = "mama")]
    MAMA, // MESA Adaptive Moving Average (MAMA) 梅萨自适应移动平均线
    #[strum(serialize = "t3")]
    T3, // Triple Exponential Moving Average (T3) 三重指数移动平均线
}

// 1. 定义trait
pub trait IndicatorConfigTrait {
    fn new(config: &Value) -> Result<Self, String> where Self: Sized; // 创建指标配置,有可能失败
}

// 2. 为枚举使用enum_dispatch
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "indicator_type", content = "indicator_config")]
pub enum IndicatorConfig {
    // 简单移动平均线
    #[serde(rename = "sma")]
    MA(MAConfig),

    #[serde(rename = "macd")]
    MACD(MACDConfig),

    #[serde(rename = "bbands")]
    BBands(BBandsConfig),

    #[serde(rename = "rsi")]
    RSI(RSIConfig),

    #[serde(rename = "adx")]
    ADX(ADXConfig),
}

impl_indicator_config!(IndicatorConfig,
    (MA,MACD,BBands,RSI,ADX)
);

// // 使用宏自动生成ToString实现
// impl ToString for IndicatorConfig {
//     fn to_string(&self) -> String {
//         match self {
//             IndicatorConfig::MA(config) => config.to_string(),
//             IndicatorConfig::MACD(config) => config.to_string(),
//             IndicatorConfig::BBands(config) => config.to_string(),
//             IndicatorConfig::RSI(config) => config.to_string(),
//             IndicatorConfig::ADX(config) => config.to_string(),
//         }
//     }
// }

// impl FromStr for IndicatorConfig {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         // 提取指标类型（如"sma"）
//         let indicator_type = if s.contains("(") {
//             s.split("(").next().unwrap_or("").trim()
//         } else {
//             s
//         };

//         // 根据指标类型创建相应的配置
//         match indicator_type {
//             "ma" => Ok(IndicatorConfig::MA(MAConfig::from_str(s)?)),
//             "macd" => Ok(IndicatorConfig::MACD(MACDConfig::from_str(s)?)),
//             "bbands" => Ok(IndicatorConfig::BBands(BBandsConfig::from_str(s)?)),
//             "rsi" => Ok(IndicatorConfig::RSI(RSIConfig::from_str(s)?)),
//             "adx" => Ok(IndicatorConfig::ADX(ADXConfig::from_str(s)?)),
//             _ => Err(format!("不支持的指标类型: {}", indicator_type)),
//         }
//     }
// }

// impl IndicatorConfig {
//     pub fn new(indicator_type: &str, config: &Value) -> Result<Self, String> {
//         match indicator_type {
//             "ma" => Ok(IndicatorConfig::MA(MAConfig::new(config)?)),
//             "macd" => Ok(IndicatorConfig::MACD(MACDConfig::new(config)?)),
//             "bbands" => Ok(IndicatorConfig::BBands(BBandsConfig::new(config)?)),
//             "rsi" => Ok(IndicatorConfig::RSI(RSIConfig::new(config)?)),
//             "adx" => Ok(IndicatorConfig::ADX(ADXConfig::new(config)?)),
//             _ => Err(format!("不支持的指标类型: {}", indicator_type)),
//         }
//     }
// }





pub trait IndicatorTrait {
    fn to_json(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn to_json_with_time(&self) -> serde_json::Value;
}


#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub enum Indicator {
    MA(MA),
    BBands(BBands),
    MACD(MACD),
    RSI(RSI),
    ADX(ADX),
}

// 使用宏自动生成所有重复的match实现
impl_indicator!(Indicator, MA, BBands, MACD, RSI, ADX);

// 删除手动的trait实现，因为宏会自动生成
// impl IndicatorTrait for Indicator {
//     fn to_json(&self) -> serde_json::Value {
//         match self {
//             Indicator::MA(ma) => IndicatorTrait::to_json(ma),
//             Indicator::BBands(bbands) => IndicatorTrait::to_json(bbands),
//             Indicator::MACD(macd) => IndicatorTrait::to_json(macd),
//             Indicator::RSI(rsi) => IndicatorTrait::to_json(rsi),
//         }
//     }

//     fn to_list(&self) -> Vec<f64> {
//         match self {
//             Indicator::MA(ma) => IndicatorTrait::to_list(ma),
//             Indicator::BBands(bbands) => IndicatorTrait::to_list(bbands),
//             Indicator::MACD(macd) => IndicatorTrait::to_list(macd),
//             Indicator::RSI(rsi) => IndicatorTrait::to_list(rsi),
//         }
//     }

//     fn to_json_with_time(&self) -> serde_json::Value {
//         match self {
//             Indicator::MA(ma) => IndicatorTrait::to_json_with_time(ma),
//             Indicator::BBands(bbands) => IndicatorTrait::to_json_with_time(bbands),
//             Indicator::MACD(macd) => IndicatorTrait::to_json_with_time(macd),
//             Indicator::RSI(rsi) => IndicatorTrait::to_json_with_time(rsi),
//         }
//     }
// }

impl From<Indicator> for CacheValue {   
    fn from(indicator: Indicator) -> Self {
        CacheValue::Indicator(indicator)
    }
}

// impl CacheItem for Indicator {
//     fn get_timestamp(&self) -> i64 {
//         match self {
//             Indicator::MA(ma) => ma.timestamp,
//             Indicator::BBands(bbands) => bbands.timestamp,
//             Indicator::MACD(macd) => macd.timestamp,
//             Indicator::RSI(rsi) => rsi.timestamp,
//         }
//     }

//     fn to_json(&self) -> serde_json::Value {

