use crate::Event;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use star_river_core::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};
use star_river_core::system::DateTimeUtc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum MarketEvent {
    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent),

    #[strum(serialize = "kline-series-update-event")]
    #[serde(rename = "kline-series-update-event")]
    KlineSeriesUpdate(KlineSeriesUpdateEvent),

    #[strum(serialize = "ticker-price-update-event")]
    #[serde(rename = "ticker-price-update-event")]
    TickerPriceUpdate(TickerPriceUpdateEvent),
}

impl From<MarketEvent> for Event {
    fn from(event: MarketEvent) -> Self {
        Event::Market(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineSeriesEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub datetime: DateTimeUtc,
}

impl ExchangeKlineSeriesEventInfo {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline_series: KlineSeries) -> Self {
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
pub struct ExchangeKlineEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTimeUtc,
}

impl ExchangeKlineEventInfo {
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
pub struct KlineUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTimeUtc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub datetime: DateTimeUtc,
    pub batch_id: String,
}

impl KlineSeriesUpdateEvent {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline_series: KlineSeries) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPriceUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub datetime: DateTimeUtc,
}

impl TickerPriceUpdateEvent {
    pub fn new(exchange: Exchange, symbol: String, ticker_price: TickerPrice) -> Self {
        Self {
            exchange,
            symbol,
            ticker_price,
            datetime: Utc::now(),
        }
    }
}
