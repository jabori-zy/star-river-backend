use derive_more::From;
use star_river_core::custom_type::NodeId;
use strategy_core::communication::{
    NodeCommandTrait,
    node::{NodeCommand, NodeResponse},
};

use crate::strategy::strategy_config::BacktestStrategyConfig;

#[derive(Debug, From)]
pub enum BacktestNodeCommand {
    GetStartNodeConfig(GetStartNodeConfigCommand),
    NodeReset(NodeResetCommand),
}

impl NodeCommandTrait for BacktestNodeCommand {
    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(command) => command.node_id(),
            BacktestNodeCommand::NodeReset(command) => command.node_id(),
        }
    }
}

// ============ Get Start Node Config ============
pub type GetStartNodeConfigCommand = NodeCommand<GetStartNodeConfigCmdPayload, GetStartNodeConfigRespPayload>;
pub type GetStartNodeConfigResponse = NodeResponse<GetStartNodeConfigRespPayload>;
// ============ Node Reset ============
pub type NodeResetCommand = NodeCommand<NodeResetCmdPayload, NodeResetRespPayload>;
pub type NodeResetResponse = NodeResponse<NodeResetRespPayload>;

// ============ Get Start Node Config ============
#[derive(Debug, From)]
pub struct GetStartNodeConfigCmdPayload;

#[derive(Debug, From)]
pub struct GetStartNodeConfigRespPayload {
    pub backtest_strategy_config: BacktestStrategyConfig,
}

impl GetStartNodeConfigRespPayload {
    pub fn new(backtest_strategy_config: BacktestStrategyConfig) -> Self {
        Self { backtest_strategy_config }
    }
}

// ============ Node Reset ============
#[derive(Debug)]
pub struct NodeResetCmdPayload;
#[derive(Debug)]
pub struct NodeResetRespPayload;
