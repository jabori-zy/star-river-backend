pub mod cache_engine;
pub mod exchange_engine;
pub mod indicator_engine;
pub mod market_engine;

use crate::communication::{Command, Response};
use cache_engine::CacheEngineCommand;
use chrono::Utc;
use derive_more::From;
use exchange_engine::ExchangeEngineCommand;
use indicator_engine::IndicatorEngineCommand;
use market_engine::MarketEngineCommand;
use star_river_core::engine::EngineName;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::system::DateTimeUtc;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
pub type EngineCommandSender = mpsc::Sender<EngineCommand>; // 命令发送器
pub type EngineCommandReceiver = mpsc::Receiver<EngineCommand>; // 命令接收器

// ================================ Engine Command Base ================================
#[derive(Debug)]
pub struct EngineCommandBase<S> {
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: oneshot::Sender<EngineResponse<S>>,
}

#[derive(Debug)]
pub struct GenericEngineCommand<T, S> {
    pub command_base: EngineCommandBase<S>,
    pub command_payload: Option<T>,
}

impl<T, S> GenericEngineCommand<T, S> {
    pub fn new(sender: String, responder: oneshot::Sender<EngineResponse<S>>, command_payload: Option<T>) -> Self {
        let command_base = EngineCommandBase {
            sender,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn sender(&self) -> String {
        self.command_base.sender.clone()
    }
}

impl<T, S> Command for GenericEngineCommand<T, S> {
    type Response = EngineResponse<S>;

    fn datetime(&self) -> DateTimeUtc {
        self.command_base.datetime
    }

    fn respond(self, response: Self::Response) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for GenericEngineCommand<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self
            .command_payload
            .as_ref()
            .expect("Command payload should exist when accessing data")
    }
}

// ================================ Engine Response Base ================================
#[derive(Debug)]
pub struct EngineResponseBase {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait + Send + Sync>>,
    pub datetime: DateTimeUtc,
}

#[derive(Debug)]
pub struct EngineResponse<S> {
    pub response_base: EngineResponseBase,
    pub response_payload: Option<S>,
}

impl EngineResponseBase {
    pub fn success() -> Self {
        Self {
            success: true,
            error: None,
            datetime: Utc::now(),
        }
    }

    pub fn error(error: Arc<dyn StarRiverErrorTrait + Send + Sync>) -> Self {
        Self {
            success: false,
            error: Some(error),
            datetime: Utc::now(),
        }
    }
}

impl<S> EngineResponse<S> {
    pub fn success(response_payload: Option<S>) -> Self {
        Self {
            response_base: EngineResponseBase::success(),
            response_payload,
        }
    }

    pub fn error(error: Arc<dyn StarRiverErrorTrait + Send + Sync>) -> Self {
        Self {
            response_base: EngineResponseBase::error(error),
            response_payload: None,
        }
    }
}

impl<S> Response for EngineResponse<S> {
    fn is_success(&self) -> bool {
        self.response_base.success
    }

    fn get_error(&self) -> Arc<dyn StarRiverErrorTrait + Send + Sync> {
        self.response_base.error.clone().expect("Error should exist when success is false")
    }

    fn datetime(&self) -> DateTimeUtc {
        self.response_base.datetime
    }
}

impl<S> Deref for EngineResponse<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self
            .response_payload
            .as_ref()
            .expect("Response payload should exist when accessing data")
    }
}

#[derive(Debug, From)]
pub enum EngineCommand {
    #[cfg(feature = "paid")]
    CacheEngine(CacheEngineCommand),         // 缓存引擎命令
    IndicatorEngine(IndicatorEngineCommand), // 指标引擎命令
    ExchangeEngine(ExchangeEngineCommand),   // 交易所引擎命令
    MarketEngine(MarketEngineCommand),       // 市场引擎命令
}

impl EngineCommand {
    pub fn get_engine_name(&self) -> EngineName {
        match self {
            #[cfg(feature = "paid")]
            EngineCommand::CacheEngine(_) => EngineName::CacheEngine,
            EngineCommand::IndicatorEngine(_) => EngineName::IndicatorEngine,
            EngineCommand::ExchangeEngine(_) => EngineName::ExchangeEngine,
            EngineCommand::MarketEngine(_) => EngineName::MarketEngine,
        }
    }
}
