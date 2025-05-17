pub mod cache_engine_response;
pub mod exchange_engine_response;
pub mod market_engine_response;
pub mod indicator_engine_response;

use cache_engine_response::CacheEngineResponse;
use exchange_engine_response::ExchangeEngineResponse;
use indicator_engine_response::IndicatorEngineResponse;
use market_engine_response::MarketEngineResponse;


pub trait ResponseTrait {
    fn code(&self) -> i32;
    fn message(&self) -> String;
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
    pub fn code(&self) -> i32 {
        match self {
            Response::CacheEngine(response) => response.code(),
            Response::IndicatorEngine(response) => response.code(),
            Response::MarketEngine(response) => response.code(),
            Response::ExchangeEngine(response) => response.code(),
        }
    }

    pub fn message(&self) -> String {
        match self {
            Response::CacheEngine(response) => response.message(),
            Response::IndicatorEngine(response) => response.message(),
            Response::MarketEngine(response) => response.message(),
            Response::ExchangeEngine(response) => response.message(),
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














