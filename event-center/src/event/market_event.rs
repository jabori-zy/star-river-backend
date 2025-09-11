use crate::Event;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use star_river_core::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};
use strum::Display;
use utils::get_utc8_datetime;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum MarketEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineInfo),

    #[strum(serialize = "kline-series-update")]
    #[serde(rename = "kline-series-update")]
    KlineSeriesUpdate(KlineSeriesInfo),

    #[strum(serialize = "ticker-price-update")]
    #[serde(rename = "ticker-price-update")]
    TickerPriceUpdate(TickerPriceInfo),
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
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeKlineSeriesEventInfo {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        kline_series: KlineSeries,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline_series,
            datetime: get_utc8_datetime(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeKlineEventInfo {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline: Kline) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline,
            datetime: get_utc8_datetime(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub datetime: DateTime<FixedOffset>,
    pub batch_id: String,
}

impl KlineSeriesInfo {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        kline_series: KlineSeries,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline_series,
            datetime: get_utc8_datetime(),
            batch_id: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPriceInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub datetime: DateTime<FixedOffset>,
}

impl TickerPriceInfo {
    pub fn new(exchange: Exchange, symbol: String, ticker_price: TickerPrice) -> Self {
        Self {
            exchange,
            symbol,
            ticker_price,
            datetime: get_utc8_datetime(),
        }
    }
}
