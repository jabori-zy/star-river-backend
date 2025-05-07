use serde::{Deserialize, Serialize};
use types::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};
use strum::Display;
use crate::Event;



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
    pub event_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineUpdateEvent {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub event_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTickerPriceUpdateEvent{
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub event_timestamp: i64,
}
