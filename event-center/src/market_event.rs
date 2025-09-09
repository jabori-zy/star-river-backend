use crate::Event;
use serde::{Deserialize, Serialize};
use strum::Display;
use types::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};

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
    pub event_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub event_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub event_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub event_timestamp: i64,
    pub batch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPriceInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub event_timestamp: i64,
}
