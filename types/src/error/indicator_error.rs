use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;


#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorError {
    #[snafu(display("indicator type [{indicator_type}] is unsupported"))]
    UnsupportedIndicatorType {
        indicator_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display("create indicator [{indicator_type}] failed. reason: [{source}]"))]
    CreateIndicatorFailed {
        indicator_type: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl crate::error::error_trait::StarRiverErrorTrait for IndicatorError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR"
    }
    
    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IndicatorError::UnsupportedIndicatorType { .. } => 1001,
            IndicatorError::CreateIndicatorFailed { .. } => 1002,
        };
        format!("{}_{:04}", prefix, code)
    }
    
    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            IndicatorError::UnsupportedIndicatorType { .. } |
            IndicatorError::CreateIndicatorFailed { .. }
        )
    }
    
}






