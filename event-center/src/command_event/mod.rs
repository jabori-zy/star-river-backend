
pub mod indicator_engine_command;
pub mod cache_engine_command;
pub mod position_engine_command;
pub mod exchange_engine_command;
pub mod market_engine_command;
pub mod order_engine_command;


use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use crate::Event;
use position_engine_command::PositionEngineCommand;
use cache_engine_command::CacheEngineCommand;
use indicator_engine_command::IndicatorEngineCommand;
use exchange_engine_command::ExchangeEngineCommand;
use market_engine_command::MarketEngineCommand;
use order_engine_command::OrderEngineCommand;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CommandEvent {
    CacheEngine(CacheEngineCommand), // 缓存引擎命令
    IndicatorEngine(IndicatorEngineCommand), // 指标引擎命令
    ExchangeEngine(ExchangeEngineCommand), // 交易所引擎命令
    MarketEngine(MarketEngineCommand), // 市场引擎命令
    OrderEngine(OrderEngineCommand), // 订单引擎命令
    PositionEngine(PositionEngineCommand), // 仓位引擎命令
    
}

impl From<CommandEvent> for Event {
    fn from(event: CommandEvent) -> Self {
        Event::Command(event)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCommandParams {
    pub strategy_id: i32,
    pub node_id: String,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}










