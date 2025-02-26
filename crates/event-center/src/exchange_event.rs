use serde::{Deserialize, Serialize};
use types::market::{Exchange, Kline, KlineInterval, KlineSeries, TickerPrice};
use strum::Display;
use crate::Event;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
// 交易所事件
pub enum ExchangeEvent {
    #[strum(serialize = "exchange-kline-series-update")]
    #[serde(rename = "exchange-kline-series-update")]
    ExchangeKlineSeriesUpdate(ExchangeKlineSeriesUpdateEventInfo),
    #[strum(serialize = "exchange-kline-update")]
    #[serde(rename = "exchange-kline-update")]
    ExchangeKlineUpdate(ExchangeKlineUpdateEventInfo),
    #[strum(serialize = "exchange-ticker-price-update")]
    #[serde(rename = "exchange-ticker-price-update")]
    ExchangeTickerPriceUpdate(ExchangeTickerPriceUpdateEventInfo),
}

impl From<ExchangeEvent> for Event {
    fn from(event: ExchangeEvent) -> Self {
        Event::Exchange(event)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineSeriesUpdateEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_series: KlineSeries,
    pub event_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeKlineUpdateEventInfo {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline: Kline,
    pub event_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTickerPriceUpdateEventInfo{
    pub exchange: Exchange,
    pub symbol: String,
    pub ticker_price: TickerPrice,
    pub event_timestamp: i64,
}
