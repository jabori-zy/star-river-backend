use super::super::{EngineCommand, EngineCommandTrait, EngineResponder};
use chrono::Utc;
use star_river_core::market::Exchange;
use star_river_core::system::DateTimeUtc;
use std::fmt::Debug;

#[derive(Debug)]
pub enum ExchangeEngineCommand {
    RegisterExchange(RegisterExchangeParams),
    UnregisterExchange(UnregisterExchangeParams),
}

impl EngineCommandTrait for ExchangeEngineCommand {
    fn responder(&self) -> &EngineResponder {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => &params.responder,
            ExchangeEngineCommand::UnregisterExchange(params) => &params.responder,
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => params.datetime,
            ExchangeEngineCommand::UnregisterExchange(params) => params.datetime,
        }
    }

    fn sender(&self) -> String {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => params.sender.clone(),
            ExchangeEngineCommand::UnregisterExchange(params) => params.sender.clone(),
        }
    }
}

impl From<ExchangeEngineCommand> for EngineCommand {
    fn from(command: ExchangeEngineCommand) -> Self {
        EngineCommand::ExchangeEngine(command)
    }
}

#[derive(Debug)]
pub struct RegisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub exchange: Exchange,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl RegisterExchangeParams {
    pub fn new(account_id: i32, exchange: Exchange, sender: String, responder: EngineResponder) -> Self {
        Self {
            account_id,
            exchange,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<RegisterExchangeParams> for EngineCommand {
    fn from(params: RegisterExchangeParams) -> Self {
        EngineCommand::ExchangeEngine(ExchangeEngineCommand::RegisterExchange(params))
    }
}

#[derive(Debug)]
pub struct UnregisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl UnregisterExchangeParams {
    pub fn new(account_id: i32, sender: String, responder: EngineResponder) -> Self {
        Self {
            account_id,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}
