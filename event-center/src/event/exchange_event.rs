use crate::Event;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use star_river_core::market::{Exchange, Kline, KlineInterval, TickerPrice};
use star_river_core::strategy::TimeRange;
use strum::Display;
use star_river_core::utils::get_utc8_datetime;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
// 交易所事件
pub enum ExchangeEvent {
    #[strum(serialize = "exchange-kline-series-update")]
    #[serde(rename = "exchange-kline-series-update")]
    ExchangeKlineSeriesUpdate(ExchangeKlineSeriesUpdateEvent),

    #[strum(serialize = "exchange-kline-update")]
    #[serde(rename = "exchange-kline-update")]
    ExchangeKlineUpdate(ExchangeKlineUpdateEvent),

    #[strum(serialize = "exchange-ticker-price-update")]
    #[serde(rename = "exchange-ticker-price-update")]
    ExchangeTickerPriceUpdate(ExchangeTickerPriceUpdateEvent),

    #[strum(serialize = "exchange-kline-history-update")]
    #[serde(rename = "exchange-kline-history-update")]
    ExchangeKlineHistoryUpdate(ExchangeKlineHistoryUpdateEvent),
}

impl From<ExchangeEvent> for Event {
    fn from(event: ExchangeEvent) -> Self {
        Event::Exchange(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineSeriesUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: Vec<Kline>,
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeKlineSeriesUpdateEvent {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        kline_series: Vec<Kline>,
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
pub struct ExchangeKlineHistoryUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub time_range: TimeRange,
    pub kline_history: Vec<Kline>, // 历史k线
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeKlineHistoryUpdateEvent {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
        kline_history: Vec<Kline>,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            time_range,
            kline_history,
            datetime: get_utc8_datetime(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeKlineUpdateEvent {
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
pub struct ExchangeTickerPriceUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub datetime: DateTime<FixedOffset>,
}

impl ExchangeTickerPriceUpdateEvent {
    pub fn new(exchange: Exchange, symbol: String, ticker_price: TickerPrice) -> Self {
        Self {
            exchange,
            symbol,
            ticker_price,
            datetime: get_utc8_datetime(),
        }
    }
}
