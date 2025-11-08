use chrono::Utc;
use derive_more::From;
use event_center_core::event::Event;
use serde::{Deserialize, Serialize};
use star_river_core::exchange::Exchange;
use star_river_core::kline::{Kline, KlineInterval};
// use star_river_core::market::TickerPrice;
use star_river_core::system::DateTimeUtc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_name")]
pub enum MarketEvent {
    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent),

    #[strum(serialize = "kline-series-update-event")]
    #[serde(rename = "kline-series-update-event")]
    KlineSeriesUpdate(KlineSeriesUpdateEvent),

    // #[strum(serialize = "ticker-price-update-event")]
    // #[serde(rename = "ticker-price-update-event")]
    // TickerPriceUpdate(TickerPriceUpdateEvent),
}

pub type KlineUpdateEvent = Event<KlineUpdatePayload>;
pub type KlineSeriesUpdateEvent = Event<KlineSeriesUpdatePayload>;
// pub type TickerPriceUpdateEvent = Event<TickerPriceUpdatePayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTimeUtc,
}

impl KlineUpdatePayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline: Kline) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline,
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: Vec<Kline>,
    pub datetime: DateTimeUtc,
    pub batch_id: String,
}

impl KlineSeriesUpdatePayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline_series: Vec<Kline>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline_series,
            datetime: Utc::now(),
            batch_id: "".to_string(),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TickerPriceUpdatePayload {
//     pub exchange: Exchange,
//     pub symbol: String,
//     pub ticker_price: TickerPrice,
//     pub datetime: DateTimeUtc,
// }

// impl TickerPriceUpdatePayload {
//     pub fn new(exchange: Exchange, symbol: String, ticker_price: TickerPrice) -> Self {
//         Self {
//             exchange,
//             symbol,
//             ticker_price,
//             datetime: Utc::now(),
//         }
//     }
// }
