pub mod start_node_error;
pub mod futures_order_node_error;
pub mod if_else_node_error;
pub mod indicator_node_error;
pub mod kline_node_error;
pub mod position_node_error;
pub mod variable_node_error;

pub use futures_order_node_error::FuturesOrderNodeError;
pub use variable_node_error::VariableNodeError;
pub use if_else_node_error::IfElseNodeError;
pub use indicator_node_error::IndicatorNodeError;
pub use kline_node_error::KlineNodeError;
pub use position_node_error::PositionNodeError;
pub use start_node_error::StartNodeError;

use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode, generate_error_code_chain};
use snafu::{Backtrace, Snafu};

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


    #[snafu(display("kline node [{node_id}] name is null"))]
    NodeNameIsNull {
        node_id: String,
        backtrace: Backtrace,
    },

    #[snafu(display("kline node id is null"))]
    NodeIdIsNull {
        backtrace: Backtrace,
    },

    #[snafu(display("kline node [{node_id}] data is null"))]
    NodeDataIsNull {
        node_id: String,
        backtrace: Backtrace,
    },


    #[snafu(display("[{node_name}] config {config_name} should be greater than or equal to(>= 0) zero, but got {config_value}"))]
    ValueNotGreaterThanOrEqualToZero {
        node_name: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    // > 0
    #[snafu(display(
        "[{node_name}] config {config_name} should be greater than(> 0) zero, but got {config_value}"
    ))]
    ValueNotGreaterThanZero {
        node_name: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeError
impl StarRiverErrorTrait for BacktestNodeError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // Transparent errors - 委托给底层错误源
            BacktestNodeError::StartNodeError { source, .. } => source.error_code(),
            BacktestNodeError::KlineNodeError { source, .. } => source.error_code(),
            BacktestNodeError::IndicatorNodeError { source, .. } => source.error_code(),
            BacktestNodeError::IfElseNodeError { source, .. } => source.error_code(),
            BacktestNodeError::VariableNodeError { source, .. } => source.error_code(),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.error_code(),
            BacktestNodeError::PositionNodeError { source, .. } => source.error_code(),
            BacktestNodeError::StateMachineError { source, .. } => source.error_code(),

            // Non-transparent errors - use self error code
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    BacktestNodeError::UnsupportedNodeType { .. } => 1001, // unsupported backtest node type
                    BacktestNodeError::ConfigFieldValueNull { .. } => 1002, // backtest node config field value is null
                    BacktestNodeError::ConfigDeserializationFailed { .. } => 1003, // backtest node config deserialization failed
                    BacktestNodeError::NodeNameIsNull { .. } => 1004, // backtest node name is null
                    BacktestNodeError::NodeIdIsNull { .. } => 1005, // backtest node id is null
                    BacktestNodeError::NodeDataIsNull { .. } => 1006, // backtest node data is null
                    BacktestNodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1007, // value not greater than or equal to zero (>= 0)
                    BacktestNodeError::ValueNotGreaterThanZero { .. } => 1008, // value not greater than zero (> 0)
                    _ => unreachable!("All transparent errors should be handled above"),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }


    fn http_status_code(&self) -> StatusCode {
        match self {
            BacktestNodeError::StateMachineError { source, .. } => source.http_status_code(),
            BacktestNodeError::StartNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::KlineNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::UnsupportedNodeType { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            BacktestNodeError::IndicatorNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::IfElseNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::VariableNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::PositionNodeError { source, .. } => source.http_status_code(),
            BacktestNodeError::ConfigFieldValueNull { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            BacktestNodeError::ConfigDeserializationFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            BacktestNodeError::NodeNameIsNull { .. } => StatusCode::BAD_REQUEST,
            BacktestNodeError::NodeIdIsNull { .. } => StatusCode::BAD_REQUEST,
            BacktestNodeError::NodeDataIsNull { .. } => StatusCode::BAD_REQUEST,
            BacktestNodeError::ValueNotGreaterThanOrEqualToZero { .. } => StatusCode::BAD_REQUEST,
            BacktestNodeError::ValueNotGreaterThanZero { .. } => StatusCode::BAD_REQUEST,
        }
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
                    BacktestNodeError::NodeNameIsNull { node_id, .. } => {
                        format!("回测节点 [{node_id}] 名称是空")
                    }
                    BacktestNodeError::NodeIdIsNull { .. } => {
                        format!("回测节点 id 是空")
                    }
                    BacktestNodeError::NodeDataIsNull { node_id, .. } => {
                        format!("回测节点 [{node_id}] 数据是空")
                    }
                    BacktestNodeError::ValueNotGreaterThanOrEqualToZero { node_name, config_name, config_value, .. } => {
                        format!("[{node_name}] 配置 {config_name} 应该大于等于零(>= 0)，但值为 {config_value}")
                    }
                    BacktestNodeError::ValueNotGreaterThanZero { node_name, config_name, config_value, .. } => {
                        format!("[{node_name}] 配置 {config_name} 应该大于零(> 0)，但值为 {config_value}")
                    }
                }
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // transparent errors - delegate to source
            BacktestNodeError::StateMachineError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::StartNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::KlineNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::IndicatorNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::IfElseNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::VariableNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::FuturesOrderNodeError { source, .. } => generate_error_code_chain(source),
            BacktestNodeError::PositionNodeError { source, .. } => generate_error_code_chain(source),

            // non-transparent errors - return own error code
            _ => vec![self.error_code()],
        }
    }
}
