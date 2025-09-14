pub mod backtest_strategy;

pub use backtest_strategy::*;

use tokio::sync::{mpsc, oneshot};
use star_river_core::system::DateTimeUtc;
use star_river_core::custom_type::NodeId;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use std::sync::Arc;

pub type StrategyResponder = oneshot::Sender<StrategyResponse>;
pub type StrategyCommandSender = mpsc::Sender<StrategyCommand>;
pub type StrategyCommandReceiver = mpsc::Receiver<StrategyCommand>;

pub trait StrategyCommandTrait {
    fn node_id(&self) -> &NodeId;
    fn responder(&self) -> &StrategyResponder;
    fn datetime(&self) -> DateTimeUtc;
}

#[derive(Debug)]
pub enum StrategyCommand {
    BacktestStrategy(BacktestStrategyCommand),
}

impl StrategyCommand {
    pub fn node_id(&self) -> &NodeId {
        match self {
            StrategyCommand::BacktestStrategy(command) => command.node_id(),
        }
    }

    pub fn responder(&self) -> &StrategyResponder {
        match self {
            StrategyCommand::BacktestStrategy(command) => command.responder(),
        }
    }

    pub fn datetime(&self) -> DateTimeUtc {
        match self {
            StrategyCommand::BacktestStrategy(command) => command.datetime(),
        }
    }
}

pub trait StrategyResponseTrait {
    fn node_id(&self) -> &NodeId;
    fn success(&self) -> bool;
    fn error(&self) -> Arc<dyn StarRiverErrorTrait>;
    fn datetime(&self) -> DateTimeUtc;
}

pub type NodeResponder = oneshot::Sender<NodeResponse>;
pub type NodeCommandSender = mpsc::Sender<NodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<NodeCommand>;

#[derive(Debug)]
pub enum StrategyResponse {
    BacktestStrategy(BacktestStrategyResponse),
}

impl StrategyResponse {
    pub fn success(&self) -> bool {
        match self {
            StrategyResponse::BacktestStrategy(response) => response.success(),
        }
    }

    pub fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            StrategyResponse::BacktestStrategy(response) => response.error(),
        }
    }

    pub fn datetime(&self) -> DateTimeUtc {
        match self {
            StrategyResponse::BacktestStrategy(response) => response.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            StrategyResponse::BacktestStrategy(response) => response.node_id(),
        }
    }
}

pub trait NodeCommandTrait {
    fn responder(&self) -> &NodeResponder;
    fn datetime(&self) -> DateTimeUtc;
    fn node_id(&self) -> &NodeId;
}

pub enum NodeCommand {
    BacktestNode(BacktestNodeCommand),
}

impl NodeCommand {
    pub fn responder(&self) -> &NodeResponder {
        match self {
            NodeCommand::BacktestNode(command) => command.responder(),
        }
    }

    pub fn datetime(&self) -> DateTimeUtc {
        match self {
            NodeCommand::BacktestNode(command) => command.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            NodeCommand::BacktestNode(command) => command.node_id(),
        }
    }
}

pub trait NodeResponseTrait {
    fn success(&self) -> bool;
    fn error(&self) -> Arc<dyn StarRiverErrorTrait>;
    fn datetime(&self) -> DateTimeUtc;
}

#[derive(Debug)]
pub enum NodeResponse {
    BacktestNode(BacktestNodeResponse),
}

impl NodeResponse {
    pub fn success(&self) -> bool {
        match self {
            NodeResponse::BacktestNode(response) => response.success(),
        }
    }

    pub fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            NodeResponse::BacktestNode(response) => response.error(),
        }
    }

    pub fn datetime(&self) -> DateTimeUtc {
        match self {
            NodeResponse::BacktestNode(response) => response.datetime(),
        }
    }
}
