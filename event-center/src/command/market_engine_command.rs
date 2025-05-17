use std::fmt::Debug;
use types::market::{Exchange, KlineInterval};
use types::custom_type::{StrategyId, AccountId};
use crate::{command::Command, Responder};
use super::CommandTrait;

#[derive(Debug)]
pub enum MarketEngineCommand {
    SubscribeKlineStream(SubscribeKlineStreamParams),
    UnsubscribeKlineStream(UnsubscribeKlineStreamParams),
}

impl CommandTrait for MarketEngineCommand {
    fn responder(&self) -> &Responder {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => &params.responder,
            MarketEngineCommand::UnsubscribeKlineStream(params) => &params.responder,
        }
    }
    fn timestamp(&self) -> i64 {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => params.timestamp,
            MarketEngineCommand::UnsubscribeKlineStream(params) => params.timestamp,
        }
    }
    fn sender(&self) -> String {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => params.sender.clone(),
            MarketEngineCommand::UnsubscribeKlineStream(params) => params.sender.clone(),
        }
    }
}


impl From<MarketEngineCommand> for Command {
    fn from(command: MarketEngineCommand) -> Self {
        Command::MarketEngine(command)
    }
}



#[derive(Debug)]
pub struct SubscribeKlineStreamParams {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub cache_size: u32,
    pub sender: String,
    pub timestamp:i64,
    pub responder: Responder,
}


#[derive(Debug)]
pub struct UnsubscribeKlineStreamParams {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub sender: String,
    pub timestamp:i64,
    pub responder: Responder,
}