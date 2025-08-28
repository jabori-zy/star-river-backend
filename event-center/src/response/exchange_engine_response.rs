use reqwest::dns::Resolving;
use types::market::Exchange;
use crate::response::{Response, ResponseTrait};
use types::error::ExchangeEngineError;
use utils::get_utc8_timestamp;

#[derive(Debug)]
pub enum ExchangeEngineResponse {
    RegisterExchange(RegisterExchangeResponse),
}

impl ResponseTrait for ExchangeEngineResponse {
    fn code(&self) -> i32 {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.code,
        }
    }

    fn message(&self) -> String {
        match self {
            ExchangeEngineResponse::RegisterExchange(response) => response.message.clone(),
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
    pub code: i32,
    pub success: bool,
    pub message: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub error: Option<ExchangeEngineError>,
    pub response_timestamp: i64,
}

impl From<RegisterExchangeResponse> for Response {
    fn from(response: RegisterExchangeResponse) -> Self {
        Response::ExchangeEngine(ExchangeEngineResponse::RegisterExchange(response))
    }
}

impl RegisterExchangeResponse {
    pub fn success(message: impl Into<String>, account_id: i32, exchange: Exchange) -> Self {
        Self {
            code: 0,
            success: true,
            message: message.into(),
            account_id,
            exchange,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }

    pub fn error(message: impl Into<String>, account_id: i32, exchange: Exchange, error: ExchangeEngineError) -> Self {
        Self {
            code: -1,
            success: false,
            message: message.into(),
            account_id,
            exchange,
            error: Some(error),
            response_timestamp: get_utc8_timestamp(),
        }
    }
}
