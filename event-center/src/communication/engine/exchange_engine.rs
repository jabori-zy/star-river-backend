use super::{EngineResponse, GenericEngineCommand};
use derive_more::From;
use star_river_core::market::Exchange;

#[derive(Debug, From)]
pub enum ExchangeEngineCommand {
    RegisterExchange(RegisterExchangeCommand),
    UnregisterExchange(UnregisterExchangeCommand),
}

// ============ Command and Response Type Definitions ============
pub type RegisterExchangeCommand = GenericEngineCommand<RegisterExchangeCmdPayload, RegisterExchangeRespPayload>;
pub type RegisterExchangeResponse = EngineResponse<RegisterExchangeRespPayload>;

pub type UnregisterExchangeCommand = GenericEngineCommand<UnregisterExchangeCmdPayload, UnregisterExchangeRespPayload>;
pub type UnregisterExchangeResponse = EngineResponse<UnregisterExchangeRespPayload>;

// ============ Register Exchange Command ============
#[derive(Debug)]
pub struct RegisterExchangeCmdPayload {
    pub account_id: i32,
    pub exchange: Exchange,
}

impl RegisterExchangeCmdPayload {
    pub fn new(account_id: i32, exchange: Exchange) -> Self {
        Self { account_id, exchange }
    }
}

#[derive(Debug)]
pub struct RegisterExchangeRespPayload {
    pub account_id: i32,
    pub exchange: Exchange,
}

impl RegisterExchangeRespPayload {
    pub fn new(account_id: i32, exchange: Exchange) -> Self {
        Self { account_id, exchange }
    }
}

// ============ Unregister Exchange Command ============
#[derive(Debug)]
pub struct UnregisterExchangeCmdPayload {
    pub account_id: i32,
}

impl UnregisterExchangeCmdPayload {
    pub fn new(account_id: i32) -> Self {
        Self { account_id }
    }
}

#[derive(Debug)]
pub struct UnregisterExchangeRespPayload {
    pub account_id: i32,
}
