pub mod futures_order_node_error;
pub mod get_variable_node;
pub mod if_else_node_error;
pub mod indicator_node_error;
pub mod kline_node_error;
pub mod position_management_node_error;
pub mod start_node_error;

use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

pub use futures_order_node_error::FuturesOrderNodeError;
pub use get_variable_node::GetVariableNodeError;
pub use if_else_node_error::IfElseNodeError;
pub use indicator_node_error::IndicatorNodeError;
pub use kline_node_error::KlineNodeError;
pub use position_management_node_error::PositionManagementNodeError;
pub use start_node_error::StartNodeError;

use super::node_state_machine_error::BacktestNodeStateMachineError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyNodeError {
    #[snafu(transparent)]
    StateMachine {
        source: BacktestNodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("unsupported backtest strategy node type: {node_type}"))]
    UnsupportedNodeType { node_type: String, backtrace: Backtrace },

    #[snafu(transparent)]
    StartNode { source: StartNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    KlineNode { source: KlineNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IndicatorNode { source: IndicatorNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IfElseNode { source: IfElseNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    GetVariableNode {
        source: GetVariableNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    FuturesOrderNode {
        source: FuturesOrderNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    PositionManagementNode {
        source: PositionManagementNodeError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for BacktestStrategyNodeError {
    fn get_prefix(&self) -> &'static str {
        "STRATEGY_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // transparent errors - delegate to source error
            BacktestStrategyNodeError::StateMachine { source, .. } => source.error_code(),
            BacktestStrategyNodeError::StartNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::KlineNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::IndicatorNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::IfElseNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::GetVariableNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::FuturesOrderNode { source, .. } => source.error_code(),
            BacktestStrategyNodeError::PositionManagementNode { source, .. } => source.error_code(),

            // non-transparent errors - use own error code
            BacktestStrategyNodeError::UnsupportedNodeType { .. } => {
                let prefix = self.get_prefix();
                let code = 1004;
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            BacktestStrategyNodeError::StateMachine { .. }
                | BacktestStrategyNodeError::StartNode { .. }
                | BacktestStrategyNodeError::KlineNode { .. }
                | BacktestStrategyNodeError::UnsupportedNodeType { .. }
                | BacktestStrategyNodeError::IndicatorNode { .. }
                | BacktestStrategyNodeError::IfElseNode { .. }
                | BacktestStrategyNodeError::GetVariableNode { .. }
                | BacktestStrategyNodeError::FuturesOrderNode { .. }
                | BacktestStrategyNodeError::PositionManagementNode { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => {
                match self {
                    // transparent errors - return source message directly
                    BacktestStrategyNodeError::StateMachine { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::StartNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::KlineNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::IndicatorNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::IfElseNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::GetVariableNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::FuturesOrderNode { source, .. } => source.get_error_message(language),
                    BacktestStrategyNodeError::PositionManagementNode { source, .. } => source.get_error_message(language),

                    // non-transparent errors - use custom message
                    BacktestStrategyNodeError::UnsupportedNodeType { node_type, .. } => {
                        format!("不支持的回测策略节点类型: {}", node_type)
                    }
                }
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // transparent errors - delegate to source
            BacktestStrategyNodeError::StateMachine { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::StartNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::KlineNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::IndicatorNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::IfElseNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::GetVariableNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::FuturesOrderNode { source, .. } => source.error_code_chain(),
            BacktestStrategyNodeError::PositionManagementNode { source, .. } => source.error_code_chain(),

            // non-transparent errors - return own error code
            BacktestStrategyNodeError::UnsupportedNodeType { .. } => vec![self.error_code()],
        }
    }
}
