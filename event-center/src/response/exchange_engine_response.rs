use std::error::Error;
use types::market::Exchange;
use crate::response::{Response, ResponseTrait};
use types::error::engine_error::*;
use utils::get_utc8_timestamp;
use std::sync::Arc;

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
    

    fn error(&self) -> Arc<dyn Error + Send + Sync + 'static> {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.error.as_ref().unwrap().clone(),
        }
    }

    fn response_timestamp(&self) -> i64 {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.response_timestamp,
        }
    }
}


impl From<ExchangeEngineResponse> for Response {
    fn from(response: ExchangeEngineResponse) -> Self {
        Response::ExchangeEngine(response)
    }
}


impl TryFrom<Response> for ExchangeEngineResponse {
    type Error = String;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::ExchangeEngine(response) => Ok(response),
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
    pub error: Option<Arc<dyn Error + Send + Sync + 'static>>,
    pub response_timestamp: i64,
}

impl From<RegisterExchangeResponse> for Response {
    fn from(response: RegisterExchangeResponse) -> Self {
        Response::ExchangeEngine(ExchangeEngineResponse::RegisterExchange(response))
    }
}

impl RegisterExchangeResponse {
    pub fn success(account_id: i32, exchange: Exchange) -> Self {
        Self {
            success: true,
            account_id,
            exchange,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }

    pub fn error(account_id: i32, exchange: Exchange, error: ExchangeEngineError) -> Self {
        Self {
            success: false,
            account_id,
            exchange,
            error: Some(Arc::new(error)),
            response_timestamp: get_utc8_timestamp(),
        }
    }
}
