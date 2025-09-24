use super::super::{EngineResponse, ResponseTrait};
use chrono::Utc;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::market::{Exchange, Kline, KlineInterval};
use star_river_core::system::DateTimeUtc;
use std::sync::Arc;

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
            MarketEngineResponse::GetKlineHistory(response) => response.success,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.error.as_ref().unwrap().clone(),
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.error.as_ref().unwrap().clone(),
            MarketEngineResponse::GetKlineHistory(response) => response.error.as_ref().unwrap().clone(),
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            MarketEngineResponse::SubscribeKlineStream(response) => response.datetime,
            MarketEngineResponse::UnsubscribeKlineStream(response) => response.datetime,
            MarketEngineResponse::GetKlineHistory(response) => response.datetime,
        }
    }
}

impl From<MarketEngineResponse> for EngineResponse {
    fn from(response: MarketEngineResponse) -> Self {
        EngineResponse::MarketEngine(response)
    }
}

impl TryFrom<EngineResponse> for MarketEngineResponse {
    type Error = String;

    fn try_from(response: EngineResponse) -> Result<Self, Self::Error> {
        match response {
            EngineResponse::MarketEngine(response) => Ok(response),
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
    pub datetime: DateTimeUtc,
}

impl SubscribeKlineStreamResponse {
    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<SubscribeKlineStreamResponse> for EngineResponse {
    fn from(response: SubscribeKlineStreamResponse) -> Self {
        EngineResponse::MarketEngine(MarketEngineResponse::SubscribeKlineStream(response))
    }
}

#[derive(Debug)]
pub struct UnsubscribeKlineStreamResponse {
    pub success: bool,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl UnsubscribeKlineStreamResponse {
    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<UnsubscribeKlineStreamResponse> for EngineResponse {
    fn from(response: UnsubscribeKlineStreamResponse) -> Self {
        EngineResponse::MarketEngine(MarketEngineResponse::UnsubscribeKlineStream(response))
    }
}

#[derive(Debug)]
pub struct GetKlineHistoryResponse {
    pub success: bool,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_history: Vec<Kline>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetKlineHistoryResponse {
    pub fn success(exchange: Exchange, symbol: String, interval: KlineInterval, kline_history: Vec<Kline>) -> Self {
        Self {
            success: true,
            exchange,
            symbol,
            interval,
            kline_history,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<GetKlineHistoryResponse> for EngineResponse {
    fn from(response: GetKlineHistoryResponse) -> Self {
        EngineResponse::MarketEngine(MarketEngineResponse::GetKlineHistory(response))
    }
}
