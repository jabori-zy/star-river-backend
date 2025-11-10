pub mod futures_order_node_error;
pub mod if_else_node_error;
pub mod indicator_node_error;
pub mod kline_node_error;
pub mod position_node_error;
pub mod start_node_error;
pub mod variable_node_error;

pub use futures_order_node_error::FuturesOrderNodeError;
pub use if_else_node_error::IfElseNodeError;
pub use indicator_node_error::IndicatorNodeError;
pub use kline_node_error::KlineNodeError;
pub use position_node_error::PositionNodeError;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain};
pub use start_node_error::StartNodeError;
use strategy_core::error::{NodeError, NodeStateMachineError};
pub use variable_node_error::VariableNodeError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    StateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    StartNodeError { source: StartNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    KlineNodeError { source: KlineNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IndicatorNodeError { source: IndicatorNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    IfElseNodeError { source: IfElseNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    VariableNodeError { source: VariableNodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    FuturesOrderNodeError {
        source: FuturesOrderNodeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    PositionNodeError { source: PositionNodeError, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for BacktestNodeError
impl StarRiverErrorTrait for BacktestNodeError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // All errors are transparent - delegate to source
            BacktestNodeError::NodeError { .. } => 1001,
            BacktestNodeError::StartNodeError { .. } => 1002,
            BacktestNodeError::KlineNodeError { .. } => 1003,
            BacktestNodeError::IndicatorNodeError { .. } => 1004,
            BacktestNodeError::IfElseNodeError { .. } => 1005,
            BacktestNodeError::VariableNodeError { .. } => 1006,
            BacktestNodeError::FuturesOrderNodeError { .. } => 1007,
            BacktestNodeError::PositionNodeError { .. } => 1008,
            BacktestNodeError::StateMachineError { .. } => 1009,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            // All errors are transparent - delegate to source
            BacktestNodeError::NodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::StateMachineError { source, .. } => source.http_status_code(),
            BacktestNodeError::StartNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::KlineNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::IndicatorNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::IfElseNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::VariableNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::PositionNodeError { source, .. } => source.http_status_code(),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match self {
            // All errors are transparent - delegate to source
            BacktestNodeError::NodeError { source, .. } => source.error_message(language),
            BacktestNodeError::StateMachineError { source, .. } => source.error_message(language),
            BacktestNodeError::StartNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::KlineNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::IndicatorNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::IfElseNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::VariableNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.error_message(language),
            BacktestNodeError::PositionNodeError { source, .. } => source.error_message(language),
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // All errors are transparent - delegate to source
            BacktestNodeError::NodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::StateMachineError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::StartNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::KlineNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::IndicatorNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::IfElseNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::VariableNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::PositionNodeError { source, .. } => generate_error_code_chain(source),
        }
    }
}
