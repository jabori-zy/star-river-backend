// 定义与与市场相关的类型

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use serde_json::{self, json};
use std::str::FromStr;
use serde::ser::Serializer;
use crate::cache::{CacheValue, CacheItem};
use deepsize::DeepSizeOf;
use utoipa::ToSchema;
pub type MT5Server = String;


#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash, DeepSizeOf, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    #[serde(rename = "binance")]
    Binance,
    #[serde(rename = "huobi")]
    Huobi,
    #[serde(rename = "okx")]
    Okx,
    #[serde(rename = "metatrader5")]
    Metatrader5(MT5Server),
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match self {
            Exchange::Binance => "binance".to_string(),
            Exchange::Huobi => "huobi".to_string(),
            Exchange::Okx => "okx".to_string(),
            Exchange::Metatrader5(server) => {
                if server.is_empty() {
                    "metatrader5".to_string()
                } else {
                    format!("metatrader5({})", server)
                }
            }
        }
    }
}

impl Serialize for Exchange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Exchange::Binance => serializer.serialize_str("binance"),
            Exchange::Huobi => serializer.serialize_str("huobi"),
            Exchange::Okx => serializer.serialize_str("okx"),
            Exchange::Metatrader5(server) => {
                if server.is_empty() {
                    serializer.serialize_str("metatrader5")
                } else {
                    serializer.serialize_str(&format!("metatrader5({})", server))
                }
            }
        }
    }
}


impl FromStr for Exchange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binance" => Ok(Exchange::Binance),
            "huobi" => Ok(Exchange::Huobi),
            "okx" => Ok(Exchange::Okx),
            _ => {
                // 如果是metatrader5，则需要解析出server
                if s.starts_with("metatrader5") {
                    // 检查是否使用括号格式: metatrader5(server)
                    if s.contains("(") && s.ends_with(")") {
                        let start = s.find("(").unwrap() + 1;
                        let end = s.len() - 1;
                        let server = &s[start..end];
                        Ok(Exchange::Metatrader5(server.to_string()))
                    } 
                    // 兼容原有的冒号格式: metatrader5:server
                    else if s.contains(":") {
                        let parts = s.split(":").collect::<Vec<&str>>();
                        if parts.len() > 1 {
                            Ok(Exchange::Metatrader5(parts[1].to_string()))
                        } else {
                            Ok(Exchange::Metatrader5(String::new()))
                        }
                    } 
                    // 无服务器信息的情况
                    else {
                        Ok(Exchange::Metatrader5(String::new()))
                    }
                }
                else {
                    Err(format!("无效的交易所: {}", s))
                }
            }
        }
    }
}

pub fn deserialize_exchange<'de, D>(deserializer: D) -> Result<Exchange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 首先尝试常规的反序列化
    let exchange_str = String::deserialize(deserializer)?;
    Exchange::from_str(&exchange_str).map_err(serde::de::Error::custom)
}


// k线间隔
#[derive(Clone, Serialize, Deserialize, Display, EnumString, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum KlineInterval {
    #[strum(serialize = "1m")]
    #[serde(rename = "1m")]
    Minutes1,
    #[strum(serialize = "5m")]
    #[serde(rename = "5m")]
    Minutes5,
    #[strum(serialize = "15m")]
    #[serde(rename = "15m")]
    Minutes15,
    #[strum(serialize = "30m")]
    #[serde(rename = "30m")]
    Minutes30,
    #[strum(serialize = "1h")]
    #[serde(rename = "1h")]
    Hours1,
    #[strum(serialize = "2h")]
    #[serde(rename = "2h")]
    Hours2,
    #[strum(serialize = "4h")]
    #[serde(rename = "4h")]
    Hours4,
    #[strum(serialize = "6h")]
    #[serde(rename = "6h")]
    Hours6,
    #[strum(serialize = "8h")]
    #[serde(rename = "8h")]
    Hours8,
    #[strum(serialize = "12h")]
    #[serde(rename = "12h")]
    Hours12,
    #[strum(serialize = "1d")]
    #[serde(rename = "1d")]
    Days1,
    #[strum(serialize = "1w")]
    #[serde(rename = "1w")]
    Weeks1,
    #[strum(serialize = "1M")]
    #[serde(rename = "1M")]
    Months1,
}


#[derive(Debug, Clone, Serialize, Deserialize, Display, Eq, PartialEq, Hash)]
pub enum Symbol {
    Spot(SpotSymbol),
    Futures(FuturesSymbol),
}


#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
pub enum SpotSymbol {
    #[strum(serialize = "btc-usdt-spot")]
    BTCUSDT,
    #[strum(serialize = "eth-usdt-spot")]
    ETHUSDT,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
pub enum FuturesSymbol {
    #[strum(serialize = "btc-usdt")]
    BTCUSDT,
    #[strum(serialize = "eth-usdt")]
    ETHUSDT,
}

pub trait MarketData: Serialize + Clone + Debug{
    fn to_json(&self) -> serde_json::Value;
}


#[derive(Debug, Serialize, Deserialize, Clone, DeepSizeOf)]
pub struct Kline {
    pub timestamp: i64, // 时间戳，单位为毫秒
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<Kline> for CacheValue {
    fn from(kline: Kline) -> Self {
        // 将timestamp转换为毫秒
        let timestamp = utils::seconds_to_millis(kline.timestamp);
        CacheValue::Kline(Kline { timestamp, ..kline })
    }
}

impl Kline {
    pub fn close_to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "close": self.close
            }
        )
    }

    pub fn close(&self) -> f64 {
        self.close
    }
}

impl CacheItem for Kline {
    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
    
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn to_list(&self) -> Vec<f64> {
        vec![self.timestamp as f64, self.open, self.high, self.low, self.close, self.volume]
    }
    fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": utils::timestamp_to_utc8(self.timestamp),
                "open": self.open,
                "high": self.high,
                "low": self.low,
                "close": self.close,
                "volume": self.volume
            }
        )
    }
}

impl MarketData for Kline {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineSeries {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub series: Vec<Kline>,
}

impl MarketData for KlineSeries {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}




#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TickerPrice {
    pub exchange: Exchange,
    pub symbol: String,
    pub price: f64,
    pub timestamp: i64,
}

impl MarketData for TickerPrice {
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
