pub mod cache_engine_response;
pub mod exchange_engine_response;
pub mod market_engine_response;
pub mod indicator_engine_response;
pub mod backtest_strategy_response;

use std::error::Error;
use std::sync::Arc;

use cache_engine_response::CacheEngineResponse;
use exchange_engine_response::ExchangeEngineResponse;
use indicator_engine_response::IndicatorEngineResponse;
use market_engine_response::MarketEngineResponse;
use types::error::error_trait::StarRiverErrorTrait;


pub trait ResponseTrait {
    fn success(&self) -> bool;
    fn error(&self) -> Arc<dyn StarRiverErrorTrait>;
    fn response_timestamp(&self) -> i64;
}


#[derive(Debug)]
pub enum Response {
    CacheEngine(CacheEngineResponse),
    IndicatorEngine(IndicatorEngineResponse),
    MarketEngine(MarketEngineResponse),
    ExchangeEngine(ExchangeEngineResponse),
}


impl Response {
    pub fn success(&self) -> bool {
        match self {
            Response::CacheEngine(response) => response.success(),
            Response::IndicatorEngine(response) => response.success(),
            Response::MarketEngine(response) => response.success(),
            Response::ExchangeEngine(response) => response.success(),
        }
    }

    pub fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            Response::CacheEngine(response) => response.error(),
            Response::IndicatorEngine(response) => response.error(),
            Response::MarketEngine(response) => response.error(),
            Response::ExchangeEngine(response) => response.error(),
        }
    }

    pub fn response_timestamp(&self) -> i64 {
        match self {
            Response::CacheEngine(response) => response.response_timestamp(),
            Response::IndicatorEngine(response) => response.response_timestamp(),
            Response::MarketEngine(response) => response.response_timestamp(),
            Response::ExchangeEngine(response) => response.response_timestamp(),
        }
    }
}