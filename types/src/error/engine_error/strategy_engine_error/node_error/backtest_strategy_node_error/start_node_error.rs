use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StartNodeError {

    #[snafu(display("start node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull {
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("start node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    // >= 0
    #[snafu(display("{config_name} should be greater than or equal to zero, but got {config_value}"))]
    ValueNotGreaterThanOrEqualToZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    // > 0
    #[snafu(display("{config_name} should be greater than zero, but got {config_value}"))]
    ValueNotGreaterThanZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for StartNodeError {
    fn get_prefix(&self) -> &'static str {
        "MT5"
    }
    
    fn error_code(&self) -> ErrorCode {
            let prefix = self.get_prefix();
            let code = match self {
                // HTTP and JSON errors (1001-1004)
                StartNodeError::ConfigFieldValueNull { .. } => 1001,
                StartNodeError::ConfigDeserializationFailed { .. } => 1002,
                StartNodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1003,
                StartNodeError::ValueNotGreaterThanZero { .. } => 1004,
            };   

            format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx 
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            StartNodeError::ConfigFieldValueNull { .. } |
            StartNodeError::ConfigDeserializationFailed { .. } |
            StartNodeError::ValueNotGreaterThanOrEqualToZero { .. } |
            StartNodeError::ValueNotGreaterThanZero { .. }
        )
    }
}