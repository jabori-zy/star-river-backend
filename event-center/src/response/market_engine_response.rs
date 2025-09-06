use types::market::{Exchange, KlineInterval};
use crate::response::{Response, ResponseTrait};
use utils::get_utc8_timestamp;
use std::sync::Arc;
use types::error::error_trait::StarRiverErrorTrait;


#[derive(Debug)]
pub enum MarketEngineResponse {
    SubscribeKlineStream(SubscribeKlineStreamResponse),
    UnsubscribeKlineStream(UnsubscribeKlineStreamResponse),
    GetKlineHistory(GetKlineHistoryResponse),
}

impl ResponseTrait for MarketEngineResponse {
    fn success(&self) -> bool {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.success,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.success,
            MarketEngineResponse::GetKlineHistory(response) => response.success
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.error.as_ref().unwrap().clone(),
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.error.as_ref().unwrap().clone(),
            MarketEngineResponse::GetKlineHistory(response) => response.error.as_ref().unwrap().clone(),
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



#[derive(Debug)]
pub struct SubscribeKlineStreamResponse {
    pub success: bool,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub response_timestamp: i64,
}

impl SubscribeKlineStreamResponse {
    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<SubscribeKlineStreamResponse> for Response {
    fn from(response: SubscribeKlineStreamResponse) -> Self {
        Response::MarketEngine(MarketEngineResponse::SubscribeKlineStream(response))
    }
}


#[derive(Debug)]
pub struct UnsubscribeKlineStreamResponse {
    pub success: bool,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub response_timestamp: i64,
}

impl UnsubscribeKlineStreamResponse {

    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<UnsubscribeKlineStreamResponse> for Response {
    fn from(response: UnsubscribeKlineStreamResponse) -> Self {
        Response::MarketEngine(MarketEngineResponse::UnsubscribeKlineStream(response))
    }
}


#[derive(Debug)]
pub struct GetKlineHistoryResponse {
    pub success: bool,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub response_timestamp: i64,
}

impl GetKlineHistoryResponse {
    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<GetKlineHistoryResponse> for Response {
    fn from(response: GetKlineHistoryResponse) -> Self {
        Response::MarketEngine(MarketEngineResponse::GetKlineHistory(response))
    }
}



