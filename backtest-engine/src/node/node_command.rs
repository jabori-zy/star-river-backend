use derive_more::From;
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::communication::{
    NodeCommandTrait,
    node::{NodeCommand, NodeResponse},
};

use crate::{node_catalog::futures_order_node::FuturesOrderNodeConfig, strategy::strategy_config::BacktestStrategyConfig};

#[derive(Debug, From)]
pub enum BacktestNodeCommand {
    GetStartNodeConfig(GetStartNodeConfigCommand),
    GetFuturesOrderConfig(GetFuturesOrderConfigCommand),
    NodeReset(NodeResetCommand),
}

impl NodeCommandTrait for BacktestNodeCommand {
    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(command) => command.node_id(),
            BacktestNodeCommand::GetFuturesOrderConfig(command) => command.node_id(),
            BacktestNodeCommand::NodeReset(command) => command.node_id(),
        }
    }
    fn node_name(&self) -> &NodeName {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(command) => command.node_name(),
            BacktestNodeCommand::GetFuturesOrderConfig(command) => command.node_name(),
            BacktestNodeCommand::NodeReset(command) => command.node_name(),
        }
    }
}

// ============ Get Start Node Config ============
pub type GetStartNodeConfigCommand = NodeCommand<GetStartNodeConfigCmdPayload, GetStartNodeConfigRespPayload>;
pub type GetStartNodeConfigResponse = NodeResponse<GetStartNodeConfigRespPayload>;

// ============ Get Futures Order Config ============
pub type GetFuturesOrderConfigCommand = NodeCommand<GetFuturesOrderConfigCmdPayload, GetFuturesOrderConfigRespPayload>;
pub type GetFuturesOrderConfigResponse = NodeResponse<GetFuturesOrderConfigRespPayload>;

// ============ Node Reset ============
pub type NodeResetCommand = NodeCommand<NodeResetCmdPayload, NodeResetRespPayload>;
pub type NodeResetResponse = NodeResponse<NodeResetRespPayload>;

// ============ Get Start Node Config ============
#[derive(Debug)]
pub struct GetStartNodeConfigCmdPayload;

#[derive(Debug)]
pub struct GetStartNodeConfigRespPayload {
    pub backtest_strategy_config: BacktestStrategyConfig,
}

impl GetStartNodeConfigRespPayload {
    pub fn new(backtest_strategy_config: BacktestStrategyConfig) -> Self {
        Self { backtest_strategy_config }
    }
}

#[derive(Debug)]
pub struct GetFuturesOrderConfigCmdPayload;

#[derive(Debug)]
pub struct GetFuturesOrderConfigRespPayload {
    pub futures_order_node_config: FuturesOrderNodeConfig,
}

impl GetFuturesOrderConfigRespPayload {
    pub fn new(futures_order_node_config: FuturesOrderNodeConfig) -> Self {
        Self { futures_order_node_config }
    }
}

// ============ Node Reset ============
#[derive(Debug)]
pub struct NodeResetCmdPayload;
#[derive(Debug)]
pub struct NodeResetRespPayload;
