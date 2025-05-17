
pub mod indicator_engine_command;
pub mod cache_engine_command;
pub mod exchange_engine_command;
pub mod market_engine_command;


use std::fmt::Debug;
use cache_engine_command::CacheEngineCommand;
use indicator_engine_command::IndicatorEngineCommand;
use exchange_engine_command::ExchangeEngineCommand;
use market_engine_command::MarketEngineCommand;
use types::engine::EngineName;
use crate::Responder;


pub trait CommandTrait {
    fn responder(&self) -> &Responder;
    fn timestamp(&self) -> i64;
    fn sender(&self) -> String;
}

#[derive(Debug)]
pub enum Command {
    CacheEngine(CacheEngineCommand), // 缓存引擎命令
    IndicatorEngine(IndicatorEngineCommand), // 指标引擎命令
    ExchangeEngine(ExchangeEngineCommand), // 交易所引擎命令
    MarketEngine(MarketEngineCommand), // 市场引擎命令
    
}


impl Command {
    pub fn get_engine_name(&self) -> EngineName {
        match self {
            Command::CacheEngine(_) => EngineName::CacheEngine,
            Command::IndicatorEngine(_) => EngineName::IndicatorEngine,
            Command::ExchangeEngine(_) => EngineName::ExchangeEngine,
            Command::MarketEngine(_) => EngineName::MarketEngine,
        }
    }

    pub fn sender(&self) -> String {
        match self {
            Command::CacheEngine(command) => command.sender(),
            Command::IndicatorEngine(command) => command.sender(),
            Command::ExchangeEngine(command) => command.sender(),
            Command::MarketEngine(command) => command.sender(),
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            Command::CacheEngine(command) => command.timestamp(),
            Command::IndicatorEngine(command) => command.timestamp(),
            Command::ExchangeEngine(command) => command.timestamp(),
            Command::MarketEngine(command) => command.timestamp(),
        }
    }

    pub fn responder(&self) -> &Responder {
        match self {
            Command::CacheEngine(command) => command.responder(),
            Command::IndicatorEngine(command) => command.responder(),
            Command::ExchangeEngine(command) => command.responder(),
            Command::MarketEngine(command) => command.responder(),
        }
    }
}








