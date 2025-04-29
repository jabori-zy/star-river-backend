use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::Exchange;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeEngineResponse {
    RegisterExchangeResponse(RegisterExchangeResponse),
    RegisterMt5ExchangeSuccess(RegisterMt5ExchangeSuccessResponse),
}

// 注册交易所的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExchangeResponse {
    pub code: i32,
    pub message: String,
    pub account_id: i32,
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

