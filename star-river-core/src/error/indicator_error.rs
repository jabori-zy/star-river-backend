use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorError {
    #[snafu(display("indicator type [{indicator_type}] is unsupported"))]
    UnsupportedIndicatorType { indicator_type: String, backtrace: Backtrace },

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
        matches!(
            self,
            IndicatorError::UnsupportedIndicatorType { .. } | IndicatorError::CreateIndicatorFailed { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                IndicatorError::UnsupportedIndicatorType { indicator_type, .. } => {
                    format!("不支持的指标类型 [{}]", indicator_type)
                }
                IndicatorError::CreateIndicatorFailed {
                    indicator_type, source, ..
                } => {
                    format!("创建指标 [{}] 失败，原因: [{}]", indicator_type, source)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // CreateIndicatorFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            IndicatorError::CreateIndicatorFailed { .. } => vec![self.error_code()],

            // Other errors have no source - return own error code
            _ => vec![self.error_code()],
        }
    }
}
