use serde::{Deserialize, Serialize};
use types::market::{Exchange, KlineInterval};
use crate::response::{Response, ResponseTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEngineResponse {
    SubscribeKlineStream(SubscribeKlineStreamResponse),
    UnsubscribeKlineStream(UnsubscribeKlineStreamResponse),
}

impl ResponseTrait for MarketEngineResponse {
    fn code(&self) -> i32 {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.code,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.code,
        }
    }

    fn message(&self) -> String {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.message.clone(),
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.message.clone(),
        }
    }

    fn response_timestamp(&self) -> i64 {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.response_timestamp,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.response_timestamp,
        }
    }
}


impl From<MarketEngineResponse> for Response {
    fn from(response: MarketEngineResponse) -> Self {
        Response::MarketEngine(response)
    }
}

impl TryFrom<Response> for MarketEngineResponse {
    type Error = String;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::MarketEngine(response) => Ok(response),
            _ => Err("Invalid response type".to_string()),
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineStreamResponse {
    pub code: i32,
    pub message: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeKlineStreamResponse {
    pub code: i32,
    pub message: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
}