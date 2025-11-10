use serde::{Deserialize, Serialize};
use star_river_core::kline::KlineInterval;
use strum::{Display, EnumString};

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
            BinanceKlineInterval::Months1 => KlineInterval::Months1,
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

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct BinanceKlineRaw(
    pub i64,    // 0: Open time (开盘时间)
    pub String, // 1: Open price (开盘价)
    pub String, // 2: High price (最高价)
    pub String, // 3: Low price (最低价)
    pub String, // 4: Close price (收盘价)
    pub String, // 5: Volume (成交量)
    pub i64,    // 6: Close time (收盘时间)
    pub String, // 7: Quote asset volume (成交额)
    pub i64,    // 8: Number of trades (成交笔数)
    pub String, // 9: Taker buy base asset volume (主动买入成交量)
    pub String, // 10: Taker buy quote asset volume (主动买入成交额)
    pub String, // 11: Ignore (忽略)
);

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct BinanceSymbolRaw {
    pub symbol: String,
    pub status: String,

    #[serde(rename = "baseAsset")]
    pub base_asset: String,

    #[serde(rename = "baseAssetPrecision")]
    pub base_asset_precision: u32,

    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,

    #[serde(rename = "quotePrecision")]
    pub quote_precision: u32,

    #[serde(rename = "quoteAssetPrecision")]
    pub quote_asset_precision: u32,

    #[serde(rename = "orderTypes")]
    pub order_types: Vec<String>,

    #[serde(rename = "icebergAllowed")]
    pub iceberg_allowed: bool,

    #[serde(rename = "ocoAllowed")]
    pub oco_allowed: bool,

    #[serde(rename = "otoAllowed")]
    pub oto_allowed: bool,

    #[serde(rename = "quoteOrderQtyMarketAllowed")]
    pub quote_order_qty_market_allowed: bool,

    #[serde(rename = "allowTrailingStop")]
    pub allow_trailing_stop: bool,

    #[serde(rename = "cancelReplaceAllowed")]
    pub cancel_replace_allowed: bool,

    #[serde(rename = "amendAllowed")]
    pub amend_allowed: bool,

    #[serde(rename = "pegInstructionsAllowed")]
    pub peg_instructions_allowed: bool,

    #[serde(rename = "isSpotTradingAllowed")]
    pub is_spot_trading_allowed: bool,

    #[serde(rename = "isMarginTradingAllowed")]
    pub is_margin_trading_allowed: bool,
}
