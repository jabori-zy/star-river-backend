pub mod cache_engine_response;
pub mod exchange_engine_response;
pub mod market_engine_response;
pub mod indicator_engine_response;


use serde::{Deserialize, Serialize};
use strum::Display;
use crate::Event;
use cache_engine_response::CacheEngineResponse;
use exchange_engine_response::ExchangeEngineResponse;
use indicator_engine_response::IndicatorEngineResponse;
use market_engine_response::MarketEngineResponse;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum ResponseEvent {
    CacheEngine(CacheEngineResponse),
    IndicatorEngine(IndicatorEngineResponse),
    MarketEngine(MarketEngineResponse),
    ExchangeEngine(ExchangeEngineResponse),
}

impl From<ResponseEvent> for Event {
    fn from(event: ResponseEvent) -> Self {
        Event::Response(event)
    }
}





// 指标引擎响应








