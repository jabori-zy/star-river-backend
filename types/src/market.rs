// 定义与与市场相关的类型

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use serde_json;
use std::str::FromStr;
use serde::ser::Serializer;

pub type MT5Server = String;


#[derive(Debug, Clone, Deserialize, Display, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    #[strum(serialize = "binance")]
    Binance,
    #[strum(serialize = "huobi")]
    Huobi,
    #[strum(serialize = "okx")]
    Okx,
    #[strum(serialize = "metatrader5")]
    Metatrader5(MT5Server),
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
                    serializer.serialize_str(&format!("metatrader5:{}", server))
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
                // 如果是metatrader5，则需要解析出server, 数据格式是metatrader5:server
                if s.starts_with("metatrader5") {
                    // server不一定存在，所以需要判断
                    let parts = s.split(":").collect::<Vec<&str>>();
                    if parts.len() > 1 {
                        Ok(Exchange::Metatrader5(parts[1].to_string()))
                    } else {
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


// k线间隔
#[derive(Clone, Serialize, Deserialize, Display, EnumString, Debug, Eq, PartialEq, Hash)]
pub enum KlineInterval {
    #[strum(serialize = "1m")]
    Minutes1,
    #[strum(serialize = "5m")]
    Minutes5,
    #[strum(serialize = "15m")]
    Minutes15,
    #[strum(serialize = "30m")]
    Minutes30,
    #[strum(serialize = "1h")]
    Hours1,
    #[strum(serialize = "2h")]
    Hours2,
    #[strum(serialize = "4h")]
    Hours4,
    #[strum(serialize = "6h")]
    Hours6,
    #[strum(serialize = "8h")]
    Hours8,
    #[strum(serialize = "12h")]
    Hours12,
    #[strum(serialize = "1d")]
    Days1,
    #[strum(serialize = "1w")]
    Weeks1,
    #[strum(serialize = "1M")]
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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
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
