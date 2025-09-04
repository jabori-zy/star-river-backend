pub mod kline_node_error;
pub mod start_node_error;
pub mod indicator_node_error;
pub mod if_else_node_error;
pub mod get_variable_node;
pub mod futures_order_node_error;
pub mod position_management_node_error;

use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;


pub use kline_node_error::KlineNodeError;
pub use start_node_error::StartNodeError;
pub use indicator_node_error::IndicatorNodeError;
pub use if_else_node_error::IfElseNodeError;
pub use get_variable_node::GetVariableNodeError;
pub use futures_order_node_error::FuturesOrderNodeError;
pub use position_management_node_error::PositionManagementNodeError;

use super::node_state_machine_error::BacktestNodeStateMachineError;




#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyNodeError {

    #[snafu(transparent)]
    StateMachine {
        source: BacktestNodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("unsupported node type: {node_type}"))]
    UnsupportedNodeType {
        node_type: String,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    StartNode {
        source: StartNodeError,
        backtrace: Backtrace,
    },


    #[snafu(transparent)]
    KlineNode {
        source: KlineNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    IndicatorNode {
        source: IndicatorNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    IfElseNode {
        source: IfElseNodeError,
        backtrace: Backtrace,
    },

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
    }
}


// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for BacktestStrategyNodeError {
    fn get_prefix(&self) -> &'static str {
        "STRATEGY_ENGINE"
    }
    
    fn error_code(&self) -> ErrorCode {
            let prefix = self.get_prefix();
            let code = match self {
                // HTTP and JSON errors (1001-1004)
                BacktestStrategyNodeError::StateMachine { .. } => 1001,
                BacktestStrategyNodeError::StartNode { .. } => 1002,
                BacktestStrategyNodeError::KlineNode { .. } => 1003,
                BacktestStrategyNodeError::UnsupportedNodeType { .. } => 1004,
                BacktestStrategyNodeError::IndicatorNode { .. } => 1005,
                BacktestStrategyNodeError::IfElseNode { .. } => 1006,
                BacktestStrategyNodeError::GetVariableNode { .. } => 1007,
                BacktestStrategyNodeError::FuturesOrderNode { .. } => 1008,
                BacktestStrategyNodeError::PositionManagementNode { .. } => 1009,
            };   

            format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx 
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            BacktestStrategyNodeError::StateMachine { .. } |
            BacktestStrategyNodeError::StartNode { .. } |
            BacktestStrategyNodeError::KlineNode { .. } |
            BacktestStrategyNodeError::UnsupportedNodeType { .. } |
            BacktestStrategyNodeError::IndicatorNode { .. } |
            BacktestStrategyNodeError::IfElseNode { .. } |
            BacktestStrategyNodeError::GetVariableNode { .. } |
            BacktestStrategyNodeError::FuturesOrderNode { .. } |
            BacktestStrategyNodeError::PositionManagementNode { .. }
        )
    }
}