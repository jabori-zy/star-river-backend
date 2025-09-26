use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IfElseNodeError {
    #[snafu(display("if else node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("if else node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("if else node evaluate result serialization failed. reason: {source}"))]
    EvaluateResultSerializationFailed { source: serde_json::Error, backtrace: Backtrace },
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
            IfElseNodeError::EvaluateResultSerializationFailed { .. } => 1003,
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
            IfElseNodeError::ConfigFieldValueNull { .. } | IfElseNodeError::ConfigDeserializationFailed { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All IfElseNodeError variants either have no source or
        // have external sources (serde_json::Error) that don't implement our trait
        vec![self.error_code()]
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                IfElseNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("条件判断节点回测配置字段值为空: {}", field_name)
                }
                IfElseNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("条件判断节点回测配置反序列化失败，原因: {}", source)
                }
                IfElseNodeError::EvaluateResultSerializationFailed { source, .. } => {
                    format!("条件判断节点回测评估结果序列化失败，原因: {}", source)
                }
            },
        }
    }
}
