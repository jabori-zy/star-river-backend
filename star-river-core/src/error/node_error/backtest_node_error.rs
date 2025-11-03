pub mod start_node_error;
pub mod futures_order_node_error;
pub mod if_else_node_error;
pub mod indicator_node_error;
pub mod kline_node_error;
pub mod position_node_error;
pub mod variable_node_error;

use crate::error::ErrorCode;
use crate::error::error_trait::ErrorLanguage;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

pub use futures_order_node_error::FuturesOrderNodeError;
pub use variable_node_error::VariableNodeError;
pub use if_else_node_error::IfElseNodeError;
pub use indicator_node_error::IndicatorNodeError;
pub use kline_node_error::KlineNodeError;
pub use position_node_error::PositionNodeError;
pub use start_node_error::StartNodeError;

use super::node_state_machine_error::BacktestNodeStateMachineError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestNodeError {
    #[snafu(transparent)]
    StartNodeError { source: StartNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    KlineNodeError { source: KlineNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IndicatorNodeError { source: IndicatorNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IfElseNodeError { source: IfElseNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    VariableNodeError {
        source: VariableNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    FuturesOrderNodeError {
        source: FuturesOrderNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    PositionNodeError {
        source: PositionNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    StateMachineError {
        source: BacktestNodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("unsupported backtest node type: {node_type}"))]
    UnsupportedNodeType { node_type: String, backtrace: Backtrace },

    #[snafu(display("backtest node config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("backtest node config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed { source: serde_json::Error, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for BacktestNodeError
impl crate::error::error_trait::StarRiverErrorTrait for BacktestNodeError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // transparent errors - delegate to source error
            BacktestNodeError::StartNodeError { source, .. } => source.error_code(),
            BacktestNodeError::KlineNodeError { source, .. } => source.error_code(),
            BacktestNodeError::IndicatorNodeError { source, .. } => source.error_code(),
            BacktestNodeError::IfElseNodeError { source, .. } => source.error_code(),
            BacktestNodeError::VariableNodeError { source, .. } => source.error_code(),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.error_code(),
            BacktestNodeError::PositionNodeError { source, .. } => source.error_code(),
            BacktestNodeError::StateMachineError { source, .. } => source.error_code(),

            // non-transparent errors - use own error code
            BacktestNodeError::UnsupportedNodeType { .. }  => format!("{}_{:04}", prefix, 1001), //不支持的回测节点类型
            BacktestNodeError::ConfigFieldValueNull { .. } => format!("{}_{:04}", prefix, 1002), //回测节点配置字段值为空
            BacktestNodeError::ConfigDeserializationFailed { .. } => format!("{}_{:04}", prefix, 1003), //回测节点配置反序列化失败
        };
        code
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            BacktestNodeError::StateMachineError { .. }
                | BacktestNodeError::StartNodeError { .. }
                | BacktestNodeError::KlineNodeError { .. }
                | BacktestNodeError::UnsupportedNodeType { .. }
                | BacktestNodeError::IndicatorNodeError { .. }
                | BacktestNodeError::IfElseNodeError { .. }
                | BacktestNodeError::VariableNodeError { .. }
                | BacktestNodeError::FuturesOrderNodeError { .. }
                | BacktestNodeError::PositionNodeError { .. }
                | BacktestNodeError::ConfigFieldValueNull { .. }
                | BacktestNodeError::ConfigDeserializationFailed { .. }
        )
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => {
                match self {
                    // transparent errors - return source message directly
                    BacktestNodeError::StateMachineError { source, .. } => source.error_message(language),
                    BacktestNodeError::StartNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::KlineNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::IndicatorNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::IfElseNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::VariableNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::FuturesOrderNodeError { source, .. } => source.error_message(language),
                    BacktestNodeError::PositionNodeError { source, .. } => source.error_message(language),

                    // non-transparent errors - use custom message
                    BacktestNodeError::UnsupportedNodeType { node_type, .. } => {
                        format!("不支持的回测策略节点类型: {}", node_type)
                    }
                    BacktestNodeError::ConfigFieldValueNull { field_name, .. } => {
                        format!("回测节点配置字段值为空: {}", field_name)
                    }
                    BacktestNodeError::ConfigDeserializationFailed { source, .. } => {
                        format!("回测节点配置反序列化失败，原因: {}", source)
                    }
                }
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // transparent errors - delegate to source
            BacktestNodeError::StateMachineError { source, .. } => source.error_code_chain(),
            BacktestNodeError::StartNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::KlineNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::IndicatorNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::IfElseNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::VariableNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.error_code_chain(),
            BacktestNodeError::PositionNodeError { source, .. } => source.error_code_chain(),

            // non-transparent errors - return own error code
            BacktestNodeError::UnsupportedNodeType { .. } => vec![self.error_code()],
            BacktestNodeError::ConfigFieldValueNull { .. } => vec![self.error_code()],
            BacktestNodeError::ConfigDeserializationFailed { .. } => vec![self.error_code()],
        }
    }
}
