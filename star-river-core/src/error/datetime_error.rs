use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::error_trait::StarRiverErrorTrait;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DateTimeError {
    #[snafu(display("Invalid timestamp: {timestamp}"))]
    InvalidTimestamp { timestamp: i64, backtrace: Backtrace },

    #[snafu(display("Transform timestamp failed: {timestamp}"))]
    TransformTimestampFailed { timestamp: i64, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for ExchangeClientError
impl StarRiverErrorTrait for DateTimeError {
    fn get_prefix(&self) -> &'static str {
        "DATETIME"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            DateTimeError::InvalidTimestamp { .. } => 1001,
            DateTimeError::TransformTimestampFailed { .. } => 1002,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        match self {
            DateTimeError::InvalidTimestamp { .. } => HashMap::new(),
            DateTimeError::TransformTimestampFailed { .. } => HashMap::new(),
        }
    }

    fn is_recoverable(&self) -> bool {
        match self {
            // For nested errors, delegate to the inner error's recoverability
            DateTimeError::InvalidTimestamp { .. } => false,

            // Recoverable errors (network, connection, temporary issues, trading operations)
            _ => matches!(
                self,
                // Network-related errors are usually recoverable
                DateTimeError::InvalidTimestamp { .. } | DateTimeError::TransformTimestampFailed { .. }
            ),
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                DateTimeError::InvalidTimestamp { timestamp, .. } => {
                    format!("非法时间戳: {}", timestamp)
                }
                DateTimeError::TransformTimestampFailed { timestamp, .. } => {
                    format!("时间戳转换失败: {}", timestamp)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // transparent errors - delegate to source
            DateTimeError::InvalidTimestamp { .. } => vec![self.error_code()],

            // non-transparent errors - return own error code
            _ => vec![self.error_code()],
        }
    }
}
