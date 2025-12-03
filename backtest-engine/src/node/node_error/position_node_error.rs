use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};
use strategy_core::error::{NodeError, NodeStateMachineError};
use virtual_trading::error::VtsError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum PositionNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    VtsError { source: VtsError, backtrace: Backtrace },

    #[snafu(transparent)]
    NodeStateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] position operation config not found for config id: {config_id}"))]
    OperationConfigNotFound {
        node_name: NodeName,
        config_id: i32,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] symbol not configured for position operation: {op}"))]
    SymbolNotConfigured { node_name: NodeName, op: String },
}

impl StarRiverErrorTrait for PositionNodeError {
    fn get_prefix(&self) -> &'static str {
        "POSITION_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            PositionNodeError::NodeError { .. } => 1001,             // node error
            PositionNodeError::VtsError { .. } => 1002,              // vts error
            PositionNodeError::NodeStateMachineError { .. } => 1003, // node state machine error
            PositionNodeError::OperationConfigNotFound { .. } => 1004,
            PositionNodeError::SymbolNotConfigured { .. } => 1005,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            PositionNodeError::NodeError { source, .. } => source.http_status_code(),
            PositionNodeError::VtsError { source, .. } => source.http_status_code(),
            PositionNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            PositionNodeError::OperationConfigNotFound { .. } => StatusCode::BAD_REQUEST,
            PositionNodeError::SymbolNotConfigured { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            PositionNodeError::NodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            PositionNodeError::VtsError { source, .. } => generate_error_code_chain(source, self.error_code()),
            PositionNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            PositionNodeError::OperationConfigNotFound { .. } => vec![self.error_code()],
            PositionNodeError::SymbolNotConfigured { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                PositionNodeError::NodeError { source, .. } => source.error_message(language),
                PositionNodeError::VtsError { source, .. } => source.error_message(language),
                PositionNodeError::NodeStateMachineError { source, .. } => source.error_message(language),
                PositionNodeError::OperationConfigNotFound { node_name, config_id, .. } => {
                    format!("@[{node_name}] 仓位操作配置未找到: {config_id}")
                }
                PositionNodeError::SymbolNotConfigured { node_name, op, .. } => {
                    format!("@[{node_name}] 仓位操作未配置交易对: {op}")
                }
            },
        }
    }
}
