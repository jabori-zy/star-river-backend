use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::Exchange;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeEngineResponse {
    RegisterExchangeSuccess(RegisterExchangeSuccessResponse),
    RegisterMt5ExchangeSuccess(RegisterMt5ExchangeSuccessResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExchangeSuccessResponse {
    pub exchange: Exchange,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMt5ExchangeSuccessResponse {
    pub terminal_id: i32,
    pub exchange: Exchange,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

