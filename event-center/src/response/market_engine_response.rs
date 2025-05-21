use serde::{Deserialize, Serialize};
use types::market::{Exchange, KlineInterval, Kline};
use crate::response::{Response, ResponseTrait};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEngineResponse {
    SubscribeKlineStream(SubscribeKlineStreamResponse),
    UnsubscribeKlineStream(UnsubscribeKlineStreamResponse),
    GetKlineHistory(GetKlineHistoryResponse),
}

impl ResponseTrait for MarketEngineResponse {
    fn code(&self) -> i32 {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.code,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.code,
            MarketEngineResponse::GetKlineHistory(response) => response.code,
        }
    }

    fn message(&self) -> String {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.message.clone(),
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.message.clone(),
            MarketEngineResponse::GetKlineHistory(response) => response.message.clone(),
        }
    }

    fn response_timestamp(&self) -> i64 {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.response_timestamp,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.response_timestamp,
            MarketEngineResponse::GetKlineHistory(response) => response.response_timestamp,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetKlineHistoryResponse {
    pub code: i32,
    pub message: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
}
