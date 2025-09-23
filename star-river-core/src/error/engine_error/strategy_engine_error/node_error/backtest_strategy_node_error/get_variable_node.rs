use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum GetVariableNodeError {
    #[snafu(display("get variable node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("get variable node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for GetVariableNodeError
impl crate::error::error_trait::StarRiverErrorTrait for GetVariableNodeError {
    fn get_prefix(&self) -> &'static str {
        "GET_VARIABLE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            GetVariableNodeError::ConfigFieldValueNull { .. } => 1001,
            GetVariableNodeError::ConfigDeserializationFailed { .. } => 1002,
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
            GetVariableNodeError::ConfigFieldValueNull { .. }
                | GetVariableNodeError::ConfigDeserializationFailed { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All GetVariableNodeError variants either have no source or
        // have external sources (serde_json::Error) that don't implement our trait
        vec![self.error_code()]
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                GetVariableNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("获取变量节点回测配置字段值为空: {}", field_name)
                }
                GetVariableNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("获取变量节点回测配置反序列化失败，原因: {}", source)
                }
            },
        }
    }
}
