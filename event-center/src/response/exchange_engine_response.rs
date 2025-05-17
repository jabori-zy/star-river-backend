use types::market::Exchange;
use crate::response::{Response, ResponseTrait};

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
    pub message: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub response_timestamp: i64,
}
