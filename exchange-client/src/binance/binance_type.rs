use strum::{Display, EnumString};
use serde::{Deserialize, Serialize};
use star_river_core::market::{KlineInterval, Kline};
use chrono::{TimeZone, Utc};


#[derive(Clone, Display, Serialize, Deserialize, Debug, EnumString, Eq, PartialEq, Hash)]
pub enum BinanceKlineInterval {
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

// 将KlineInterval转换为BinanceKlineInterval
impl From<BinanceKlineInterval> for KlineInterval {
    fn from(interval: BinanceKlineInterval) -> Self {
        match interval {
            BinanceKlineInterval::Minutes1 => KlineInterval::Minutes1,
            BinanceKlineInterval::Minutes5 => KlineInterval::Minutes5,
            BinanceKlineInterval::Minutes15 => KlineInterval::Minutes15,
            BinanceKlineInterval::Minutes30 => KlineInterval::Minutes30,
            BinanceKlineInterval::Hours1 => KlineInterval::Hours1,
            BinanceKlineInterval::Hours2 => KlineInterval::Hours2,
            BinanceKlineInterval::Hours4 => KlineInterval::Hours4,
            BinanceKlineInterval::Hours6 => KlineInterval::Hours6,
            BinanceKlineInterval::Hours8 => KlineInterval::Hours8,
            BinanceKlineInterval::Hours12 => KlineInterval::Hours12,
            BinanceKlineInterval::Days1 => KlineInterval::Days1,
            BinanceKlineInterval::Weeks1 => KlineInterval::Weeks1,
            BinanceKlineInterval::Months1 => KlineInterval::Months1
        }
    }
}

impl From<KlineInterval> for BinanceKlineInterval {
    fn from(interval: KlineInterval) -> Self {
        match interval {
            KlineInterval::Minutes1 => BinanceKlineInterval::Minutes1,
            KlineInterval::Minutes5 => BinanceKlineInterval::Minutes5,
            KlineInterval::Minutes15 => BinanceKlineInterval::Minutes15,
            KlineInterval::Minutes30 => BinanceKlineInterval::Minutes30,
            KlineInterval::Hours1 => BinanceKlineInterval::Hours1,
            KlineInterval::Hours2 => BinanceKlineInterval::Hours2,
            KlineInterval::Hours4 => BinanceKlineInterval::Hours4,
            KlineInterval::Hours6 => BinanceKlineInterval::Hours6,
            KlineInterval::Hours8 => BinanceKlineInterval::Hours8,
            KlineInterval::Hours12 => BinanceKlineInterval::Hours12,
            KlineInterval::Days1 => BinanceKlineInterval::Days1,
            KlineInterval::Weeks1 => BinanceKlineInterval::Weeks1,
            KlineInterval::Months1 => BinanceKlineInterval::Months1,
            _ => panic!("Invalid KlineInterval: {:?}", interval),
        }
    }
}
impl BinanceKlineInterval {
    pub const ALL: &'static [BinanceKlineInterval] = &[
        BinanceKlineInterval::Minutes1,
        BinanceKlineInterval::Minutes5,
        BinanceKlineInterval::Minutes15,
        BinanceKlineInterval::Minutes30,
        BinanceKlineInterval::Hours1,
        BinanceKlineInterval::Hours2,
        BinanceKlineInterval::Hours4,
        BinanceKlineInterval::Hours6,
        BinanceKlineInterval::Hours8,
        BinanceKlineInterval::Hours12,
        BinanceKlineInterval::Days1,
        BinanceKlineInterval::Weeks1,
        BinanceKlineInterval::Months1,
    ];

    pub fn to_list() -> &'static [BinanceKlineInterval] {
        Self::ALL
    }
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct BinanceTickerPrice {
//     pub symbol: String,
//     #[serde(deserialize_with = "deserialize_string_to_f64")]
//     pub price: f64,
//     #[serde(skip_deserializing)]
//     pub timestamp: i64,
// }

// impl From<BinanceTickerPrice> for TickerPrice {
//     fn from(ticker_price: BinanceTickerPrice) -> Self {
//         Self {
//             exchange: Exchange::Binance,
//             symbol: ticker_price.symbol,
//             price: ticker_price.price,
//             timestamp: ticker_price.timestamp,
//         }
//     }
// }




#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceKline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<BinanceKline> for Kline {
    fn from(kline: BinanceKline) -> Self {
        Self {
            datetime: Utc.timestamp_opt(kline.timestamp, 0).single().unwrap(),
            open: kline.open,
            high: kline.high,
            low: kline.low,
            close: kline.close,
            volume: kline.volume,
        }
    }
}