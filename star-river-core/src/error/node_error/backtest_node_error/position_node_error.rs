use crate::error::ErrorCode;
use crate::error::error_trait::ErrorLanguage;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum PositionNodeError {
    TestError {
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for PositionManagementNodeError
impl crate::error::error_trait::StarRiverErrorTrait for PositionNodeError {
    fn get_prefix(&self) -> &'static str {
        "POSITION_MANAGEMENT_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match *self {
            PositionNodeError::TestError { .. } => 1001,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        match *self {
            PositionNodeError::TestError { .. } => true,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match *self {
            PositionNodeError::TestError { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match *self {
                PositionNodeError::TestError { .. } => "test error".to_string(),
            },
        }
    }
}
