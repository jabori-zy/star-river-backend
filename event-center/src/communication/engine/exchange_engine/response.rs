use super::super::{EngineResponse, ResponseTrait};
use chrono::{DateTime, FixedOffset};
use star_river_core::error::engine_error::*;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::market::Exchange;
use std::sync::Arc;
use star_river_core::utils::get_utc8_datetime;

#[derive(Debug)]
pub enum ExchangeEngineResponse {
    RegisterExchange(RegisterExchangeResponse),
}

impl ResponseTrait for ExchangeEngineResponse {
    fn success(&self) -> bool {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.success,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => {
                response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.datetime,
        }
    }
}

impl From<ExchangeEngineResponse> for EngineResponse {
    fn from(response: ExchangeEngineResponse) -> Self {
        EngineResponse::ExchangeEngine(response)
    }
}

impl TryFrom<EngineResponse> for ExchangeEngineResponse {
    type Error = String;

    fn try_from(response: EngineResponse) -> Result<Self, Self::Error> {
        match response {
            EngineResponse::ExchangeEngine(response) => Ok(response),
            _ => Err("Invalid response type".to_string()),
        }
    }
}

// 注册交易所的响应
#[derive(Debug)]
pub struct RegisterExchangeResponse {
    pub success: bool,
    pub account_id: i32,
    pub exchange: Exchange,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTime<FixedOffset>,
}

impl From<RegisterExchangeResponse> for EngineResponse {
    fn from(response: RegisterExchangeResponse) -> Self {
        EngineResponse::ExchangeEngine(ExchangeEngineResponse::RegisterExchange(response))
    }
}

impl RegisterExchangeResponse {
    pub fn success(account_id: i32, exchange: Exchange) -> Self {
        Self {
            success: true,
            account_id,
            exchange,
            error: None,
            datetime: get_utc8_datetime(),
        }
    }

    pub fn error(account_id: i32, exchange: Exchange, error: ExchangeEngineError) -> Self {
        Self {
            success: false,
            account_id,
            exchange,
            error: Some(Arc::new(error)),
            datetime: get_utc8_datetime(),
        }
    }
}
