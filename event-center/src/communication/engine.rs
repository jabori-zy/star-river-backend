pub mod cache_engine;
pub mod exchange_engine;
pub mod indicator_engine;
pub mod market_engine;

use cache_engine::{CacheEngineCommand, CacheEngineResponse};
use chrono::{DateTime, FixedOffset};
use exchange_engine::{ExchangeEngineCommand, ExchangeEngineResponse};
use indicator_engine::{IndicatorEngineCommand, IndicatorEngineResponse};
use market_engine::{MarketEngineCommand, MarketEngineResponse};
use star_river_core::engine::EngineName;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub type EngineCommandSender = mpsc::Sender<EngineCommand>; // 命令发送器
pub type EngineCommandReceiver = mpsc::Receiver<EngineCommand>; // 命令接收器
pub type EngineResponder = oneshot::Sender<EngineResponse>; // 响应

pub trait EngineCommandTrait {
    fn responder(&self) -> &EngineResponder;
    fn datetime(&self) -> DateTime<FixedOffset>;
    fn sender(&self) -> String;
}

#[derive(Debug)]
pub enum EngineCommand {
    CacheEngine(CacheEngineCommand),         // 缓存引擎命令
    IndicatorEngine(IndicatorEngineCommand), // 指标引擎命令
    ExchangeEngine(ExchangeEngineCommand),   // 交易所引擎命令
    MarketEngine(MarketEngineCommand),       // 市场引擎命令
}

impl EngineCommand {
    pub fn get_engine_name(&self) -> EngineName {
        match self {
            EngineCommand::CacheEngine(_) => EngineName::CacheEngine,
            EngineCommand::IndicatorEngine(_) => EngineName::IndicatorEngine,
            EngineCommand::ExchangeEngine(_) => EngineName::ExchangeEngine,
            EngineCommand::MarketEngine(_) => EngineName::MarketEngine,
        }
    }

    pub fn sender(&self) -> String {
        match self {
            EngineCommand::CacheEngine(command) => command.sender(),
            EngineCommand::IndicatorEngine(command) => command.sender(),
            EngineCommand::ExchangeEngine(command) => command.sender(),
            EngineCommand::MarketEngine(command) => command.sender(),
        }
    }

    pub fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            EngineCommand::CacheEngine(command) => command.datetime(),
            EngineCommand::IndicatorEngine(command) => command.datetime(),
            EngineCommand::ExchangeEngine(command) => command.datetime(),
            EngineCommand::MarketEngine(command) => command.datetime(),
        }
    }

    pub fn responder(&self) -> &EngineResponder {
        match self {
            EngineCommand::CacheEngine(command) => command.responder(),
            EngineCommand::IndicatorEngine(command) => command.responder(),
            EngineCommand::ExchangeEngine(command) => command.responder(),
            EngineCommand::MarketEngine(command) => command.responder(),
        }
    }
}

pub trait ResponseTrait {
    fn success(&self) -> bool;
    fn error(&self) -> Arc<dyn StarRiverErrorTrait>;
    fn datetime(&self) -> DateTime<FixedOffset>;
}

#[derive(Debug)]
pub enum EngineResponse {
    CacheEngine(CacheEngineResponse),
    IndicatorEngine(IndicatorEngineResponse),
    MarketEngine(MarketEngineResponse),
    ExchangeEngine(ExchangeEngineResponse),
}

impl EngineResponse {
    pub fn success(&self) -> bool {
        match self {
            EngineResponse::CacheEngine(response) => response.success(),
            EngineResponse::IndicatorEngine(response) => response.success(),
            EngineResponse::MarketEngine(response) => response.success(),
            EngineResponse::ExchangeEngine(response) => response.success(),
        }
    }

    pub fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            EngineResponse::CacheEngine(response) => response.error(),
            EngineResponse::IndicatorEngine(response) => response.error(),
            EngineResponse::MarketEngine(response) => response.error(),
            EngineResponse::ExchangeEngine(response) => response.error(),
        }
    }

    pub fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            EngineResponse::CacheEngine(response) => response.datetime(),
            EngineResponse::IndicatorEngine(response) => response.datetime(),
            EngineResponse::MarketEngine(response) => response.datetime(),
            EngineResponse::ExchangeEngine(response) => response.datetime(),
        }
    }
}
