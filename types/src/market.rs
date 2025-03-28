// 定义与与市场相关的类型

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use serde_json;

#[derive(Debug,Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
pub enum Exchange {
    #[strum(serialize = "binance")]
    Binance,
    #[strum(serialize = "huobi")]
    Huobi,
    #[strum(serialize = "okx")]
    Okx,
    #[strum(serialize = "metatrader5")]
    Metatrader5,
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
