pub mod node_command;
pub mod strategy_command;

use derive_more::From;
pub use node_command::*;
pub use strategy_command::*;

use crate::communication::{Command, Response};
use chrono::Utc;
use star_river_core::custom_type::NodeId;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::system::DateTimeUtc;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub type StrategyCommandSender = mpsc::Sender<BacktestStrategyCommand>;
pub type StrategyCommandReceiver = mpsc::Receiver<BacktestStrategyCommand>;

pub type NodeCommandSender = mpsc::Sender<BacktestNodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<BacktestNodeCommand>;

// ================================ Strategy Command ================================
#[derive(Debug)]
pub struct StrategyCommandBase<S> {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: oneshot::Sender<StrategyResponse<S>>,
}

#[derive(Debug)]
pub struct StrategyCommand<T, S> {
    pub command_base: StrategyCommandBase<S>,
    pub command_payload: Option<T>,
}

impl<T, S> StrategyCommand<T, S> {
    pub fn new(node_id: NodeId, responder: oneshot::Sender<StrategyResponse<S>>, command_payload: Option<T>) -> Self {
        let command_base = StrategyCommandBase {
            node_id,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.command_base.node_id.clone()
    }
}

impl<T, S> Command for StrategyCommand<T, S> {
    type Response = StrategyResponse<S>;

    fn datetime(&self) -> DateTimeUtc {
        self.command_base.datetime
    }
    fn respond(self, response: Self::Response) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for StrategyCommand<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self
            .command_payload
            .as_ref()
            .expect("Command payload should exist when accessing data")
    }
}

#[derive(Debug)]
pub struct StrategyResponseBase {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait + Send + Sync>>,
    pub datetime: DateTimeUtc,
}

#[derive(Debug)]
pub struct StrategyResponse<S> {
    pub response_base: StrategyResponseBase,
    pub response_payload: Option<S>,
}

impl StrategyResponseBase {
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

impl<S> StrategyResponse<S> {
    pub fn success(response_payload: Option<S>) -> Self {
        Self {
            response_base: StrategyResponseBase::success(),
            response_payload,
        }
    }

    pub fn error(error: Arc<dyn StarRiverErrorTrait + Send + Sync>) -> Self {
        Self {
            response_base: StrategyResponseBase::error(error),
            response_payload: None,
        }
    }
}

impl<S> Response for StrategyResponse<S> {
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

impl<S> Deref for StrategyResponse<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self
            .response_payload
            .as_ref()
            .expect("Response payload should exist when accessing data")
    }
}

#[derive(Debug, From)]
pub enum BacktestStrategyCommand {
    GetStrategyKeys(GetStrategyKeysCommand),
    GetMinIntervalSymbols(GetMinIntervalSymbolsCommand),
    GetCurrentTime(GetCurrentTimeCommand),
    InitKlineData(InitKlineDataCommand),
    AppendKlineData(AppendKlineDataCommand),
    InitIndicatorData(InitIndicatorDataCommand),
    GetKlineData(GetKlineDataCommand),
    GetIndicatorData(GetIndicatorDataCommand),
    UpdateKlineData(UpdateKlineDataCommand),
    UpdateIndicatorData(UpdateIndicatorDataCommand),
    InitCustomVariableValue(InitCustomVariableValueCommand),
    GetCustomVariableValue(GetCustomVariableValueCommand),
    UpdateCustomVariableValue(UpdateCustomVariableValueCommand),
    ResetCustomVariableValue(ResetCustomVariableValueCommand)


}

impl BacktestStrategyCommand {
    pub fn node_id(&self) -> NodeId {
        match self {
            BacktestStrategyCommand::GetStrategyKeys(command) => command.node_id(),
            BacktestStrategyCommand::GetMinIntervalSymbols(command) => command.node_id(),
            BacktestStrategyCommand::GetCurrentTime(command) => command.node_id(),
            BacktestStrategyCommand::InitKlineData(command) => command.node_id(),
            BacktestStrategyCommand::AppendKlineData(command) => command.node_id(),
            BacktestStrategyCommand::InitIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::GetKlineData(command) => command.node_id(),
            BacktestStrategyCommand::GetIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::UpdateKlineData(command) => command.node_id(),
            BacktestStrategyCommand::UpdateIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::InitCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::GetCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::UpdateCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::ResetCustomVariableValue(command) => command.node_id(),

        }
    }
}

// ================================ Node Command ================================
#[derive(Debug)]
pub struct NodeCommandBase<S> {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: oneshot::Sender<NodeResponse<S>>,
}

#[derive(Debug)]
pub struct NodeCommand<T, S> {
    pub command_base: NodeCommandBase<S>,
    pub command_payload: Option<T>,
}

impl<T, S> NodeCommand<T, S> {
    pub fn new(node_id: NodeId, responder: oneshot::Sender<NodeResponse<S>>, command_payload: Option<T>) -> Self {
        let command_base = NodeCommandBase {
            node_id,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.command_base.node_id.clone()
    }
}

impl<T, S> Command for NodeCommand<T, S> {
    type Response = NodeResponse<S>;
    fn datetime(&self) -> DateTimeUtc {
        self.command_base.datetime
    }
    fn respond(self, response: Self::Response) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for NodeCommand<T, S> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self
            .command_payload
            .as_ref()
            .expect("Command payload should exist when accessing data")
    }
}

#[derive(Debug)]
pub struct NodeResponseBase {
    pub node_id: NodeId,
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait + Send + Sync>>,
    pub datetime: DateTimeUtc,
}

#[derive(Debug)]
pub struct NodeResponse<S> {
    pub response_base: NodeResponseBase,
    pub response_payload: Option<S>,
}

impl NodeResponseBase {
    pub fn success(node_id: NodeId) -> Self {
        Self {
            node_id,
            success: true,
            error: None,
            datetime: Utc::now(),
        }
    }

    pub fn error(node_id: NodeId, error: Arc<dyn StarRiverErrorTrait + Send + Sync>) -> Self {
        Self {
            node_id,
            success: false,
            error: Some(error),
            datetime: Utc::now(),
        }
    }
}

impl<S> NodeResponse<S> {
    pub fn success(node_id: NodeId, response_payload: Option<S>) -> Self {
        Self {
            response_base: NodeResponseBase::success(node_id),
            response_payload,
        }
    }

    pub fn error(node_id: NodeId, error: Arc<dyn StarRiverErrorTrait + Send + Sync>) -> Self {
        Self {
            response_base: NodeResponseBase::error(node_id, error),
            response_payload: None,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.response_base.node_id.clone()
    }
}

impl<S> Response for NodeResponse<S> {
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

impl<S> Deref for NodeResponse<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self
            .response_payload
            .as_ref()
            .expect("Response payload should exist when accessing data")
    }
}

pub trait NodeCommandTrait {
    // fn responder(&self) -> &NodeResponder;
    fn datetime(&self) -> DateTimeUtc;
    fn node_id(&self) -> &NodeId;
}

#[derive(Debug, From)]
pub enum BacktestNodeCommand {
    GetStartNodeConfig(GetStartNodeConfigCommand),
    NodeReset(NodeResetCommand),
}

impl BacktestNodeCommand {
    pub fn node_id(&self) -> NodeId {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(command) => command.node_id(),
            BacktestNodeCommand::NodeReset(command) => command.node_id(),
        }
    }
}
