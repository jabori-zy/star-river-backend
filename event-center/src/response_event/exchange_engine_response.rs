use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::Exchange;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeEngineResponse {
    RegisterExchangeSuccess(RegisterExchangeSuccessResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExchangeSuccessResponse {
    pub exchange: Exchange,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}
