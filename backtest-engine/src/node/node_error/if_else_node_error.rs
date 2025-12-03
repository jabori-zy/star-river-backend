use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain};
use strategy_core::error::{NodeError, NodeStateMachineError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IfElseNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    NodeStateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{node_name}] evaluate result serialization failed. reason: {source}"))]
    EvaluateResultSerializationFailed {
        node_name: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for IfElseNodeError {
    fn get_prefix(&self) -> &'static str {
        "IF_ELSE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IfElseNodeError::NodeError { .. } => 1000,                         // node error
            IfElseNodeError::NodeStateMachineError { .. } => 1001,             // node state machine error
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => 1002, // evaluate result serialization failed
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            IfElseNodeError::NodeError { source, .. } => source.http_status_code(),
            IfElseNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            IfElseNodeError::NodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            IfElseNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                IfElseNodeError::NodeError { source, .. } => source.error_message(language),
                IfElseNodeError::NodeStateMachineError { source, .. } => source.error_message(language),
                IfElseNodeError::EvaluateResultSerializationFailed { node_name, source, .. } => {
                    format!("[{node_name}] 条件结果序列化失败，原因: {source}")
                }
            },
        }
    }
}
