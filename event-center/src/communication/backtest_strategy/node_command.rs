use super::{NodeCommand, NodeResponse};
use star_river_core::custom_type::NodeId;
use star_river_core::strategy::BacktestStrategyConfig;

// ============ Get Start Node Config ============
pub type GetStartNodeConfigCommand = NodeCommand<GetStartNodeConfigCmdPayload, GetStartNodeConfigRespPayload>;
pub type GetStartNodeConfigResponse = NodeResponse<GetStartNodeConfigRespPayload>;
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

// ============ Node Reset ============
#[derive(Debug)]
pub struct NodeResetCmdPayload;
#[derive(Debug)]
pub struct NodeResetRespPayload;
