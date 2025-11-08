use chrono::Utc;
use derive_more::From;
use event_center_core::event::Event;
use serde::{Deserialize, Serialize};
use star_river_core::exchange::Exchange;
use star_river_core::kline::{Kline, KlineInterval};
use strategy_core::strategy::TimeRange;
use star_river_core::system::DateTimeUtc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
pub enum ExchangeEvent {
    #[strum(serialize = "exchange-kline-series-update")]
    #[serde(rename = "exchange-kline-series-update")]
    ExchangeKlineSeriesUpdate(ExchangeKlineSeriesUpdateEvent),

    #[strum(serialize = "exchange-kline-update")]
    #[serde(rename = "exchange-kline-update")]
    ExchangeKlineUpdate(ExchangeKlineUpdateEvent),

    // #[strum(serialize = "exchange-ticker-price-update")]
    // #[serde(rename = "exchange-ticker-price-update")]
    // ExchangeTickerPriceUpdate(ExchangeTickerPriceUpdateEvent),

    #[strum(serialize = "exchange-kline-history-update")]
    #[serde(rename = "exchange-kline-history-update")]
    ExchangeKlineHistoryUpdate(ExchangeKlineHistoryUpdateEvent),
}

pub type ExchangeKlineSeriesUpdateEvent = Event<ExchangeKlineSeriesUpdatePayload>;
pub type ExchangeKlineUpdateEvent = Event<ExchangeKlineUpdatePayload>;
// pub type ExchangeTickerPriceUpdateEvent = Event<ExchangeTickerPriceUpdatePayload>;
pub type ExchangeKlineHistoryUpdateEvent = Event<ExchangeKlineHistoryUpdatePayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineSeriesUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: Vec<Kline>,
    pub datetime: DateTimeUtc,
}

impl ExchangeKlineSeriesUpdatePayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline_series: Vec<Kline>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline_series,
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineHistoryUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub time_range: TimeRange,
    pub kline_history: Vec<Kline>,
    pub datetime: DateTimeUtc,
}

impl ExchangeKlineHistoryUpdatePayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, time_range: TimeRange, kline_history: Vec<Kline>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            time_range,
            kline_history,
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTimeUtc,
}

impl ExchangeKlineUpdatePayload {
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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExchangeTickerPriceUpdatePayload {
//     pub exchange: Exchange,
//     pub symbol: String,
//     pub ticker_price: TickerPrice,
//     pub datetime: DateTimeUtc,
// }

// impl ExchangeTickerPriceUpdatePayload {
//     pub fn new(exchange: Exchange, symbol: String, ticker_price: TickerPrice) -> Self {
//         Self {
//             exchange,
//             symbol,
//             ticker_price,
//             datetime: Utc::now(),
//         }
//     }
// }
