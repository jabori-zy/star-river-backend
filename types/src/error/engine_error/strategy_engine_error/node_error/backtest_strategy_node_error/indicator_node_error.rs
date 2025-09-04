use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use crate::error::indicator_error::IndicatorError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorNodeError {

    #[snafu(display("indicator node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull {
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("indicator node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },


    #[snafu(display("{config_name} should be greater than or equal to zero, but got {config_value}"))]
    ValueNotGreaterThanOrEqualToZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },


    #[snafu(display("{config_name} should be greater than zero, but got {config_value}"))]
    ValueNotGreaterThanZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    IndicatorError {
        source: IndicatorError,
        backtrace: Backtrace,
    },


    #[snafu(display("data source [{data_source}] parse failed. reason: [{source}]"))]
    DataSourceParseFailed {
        data_source: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    }
}

// Implement the StarRiverErrorTrait for IndicatorNodeError
impl crate::error::error_trait::StarRiverErrorTrait for IndicatorNodeError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_NODE"
    }
    
    fn error_code(&self) -> ErrorCode {
            let prefix = self.get_prefix();
            let code = match self {
                // HTTP and JSON errors (1001-1004)
                IndicatorNodeError::ConfigFieldValueNull { .. } => 1001,
                IndicatorNodeError::ConfigDeserializationFailed { .. } => 1002,
                IndicatorNodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1003,
                IndicatorNodeError::ValueNotGreaterThanZero { .. } => 1004,
                IndicatorNodeError::IndicatorError { .. } => 1005,
                IndicatorNodeError::DataSourceParseFailed { .. } => 1006,
            };   

            format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx 
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            IndicatorNodeError::ConfigFieldValueNull { .. } |
            IndicatorNodeError::ConfigDeserializationFailed { .. } |
            IndicatorNodeError::ValueNotGreaterThanOrEqualToZero { .. } |
            IndicatorNodeError::ValueNotGreaterThanZero { .. } |
            IndicatorNodeError::IndicatorError { .. } |
            IndicatorNodeError::DataSourceParseFailed { .. }
        )
    }
}