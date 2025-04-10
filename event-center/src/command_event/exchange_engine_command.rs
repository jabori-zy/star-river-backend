use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::market::Exchange;
use uuid::Uuid;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum ExchangeEngineCommand {
    #[strum(serialize = "register-exchange")]
    RegisterExchange(RegisterExchangeParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExchangeParams {
    pub exchange: Exchange,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}