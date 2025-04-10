use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEngineResponse {
    SubscribeKlineStreamSuccess(SubscribeKlineStreamSuccessResponse),
    UnsubscribeKlineStreamSuccess(UnsubscribeKlineStreamSuccessResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeKlineStreamSuccessResponse {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeKlineStreamSuccessResponse {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}