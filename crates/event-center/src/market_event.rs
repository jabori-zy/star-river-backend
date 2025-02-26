use serde::{Deserialize, Serialize};
use types::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};
use strum::Display;
use crate::Event;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum MarketEvent {
    #[strum(serialize = "exchange-kline-series-update")]
    #[serde(rename = "exchange-kline-series-update")]
    ExchangeKlineSeriesUpdate(ExchangeKlineSeriesEventInfo),
    #[strum(serialize = "exchange-kline-update")]
    #[serde(rename = "exchange-kline-update")]
    ExchangeKlineUpdate(ExchangeKlineEventInfo),
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineEventInfo),
    #[strum(serialize = "kline-series-update")]
    #[serde(rename = "kline-series-update")]
    KlineSeriesUpdate(KlineSeriesEventInfo),
    #[strum(serialize = "ticker-price-update")]
    #[serde(rename = "ticker-price-update")]
    TickerPriceUpdate(TickerPriceEventInfo),
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
pub struct KlineEventInfo{
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub event_timestamp: i64,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesEventInfo{
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub event_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPriceEventInfo{
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub event_timestamp: i64,
}


