use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IfElseNodeError {

    #[snafu(display("if else node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull {
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("if else node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    }
}

// Implement the StarRiverErrorTrait for IfElseNodeError
impl crate::error::error_trait::StarRiverErrorTrait for IfElseNodeError {
    fn get_prefix(&self) -> &'static str {
        "IF_ELSE_NODE"
    }
    
    fn error_code(&self) -> ErrorCode {
            let prefix = self.get_prefix();
            let code = match self {
                // HTTP and JSON errors (1001-1004)
                IfElseNodeError::ConfigFieldValueNull { .. } => 1001,
                IfElseNodeError::ConfigDeserializationFailed { .. } => 1002,
            };   

            format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx 
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            IfElseNodeError::ConfigFieldValueNull { .. } |
            IfElseNodeError::ConfigDeserializationFailed { .. }
        )
    }
}