use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain};
use strategy_core::error::{NodeError, NodeStateMachineError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StartNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    StateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for StartNodeError
impl StarRiverErrorTrait for StartNodeError {
    fn get_prefix(&self) -> &'static str {
        "START_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StartNodeError::NodeError { .. } => 1001,         // node error
            StartNodeError::StateMachineError { .. } => 1002, // state machine error
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            StartNodeError::NodeError { source, .. } => source.http_status_code(),
            StartNodeError::StateMachineError { source, .. } => source.http_status_code(),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                StartNodeError::NodeError { source, .. } => source.error_message(language),
                StartNodeError::StateMachineError { source, .. } => source.error_message(language),
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            StartNodeError::NodeError { source, .. } => generate_error_code_chain(source),
            StartNodeError::StateMachineError { source, .. } => generate_error_code_chain(source),
        }
    }
}
