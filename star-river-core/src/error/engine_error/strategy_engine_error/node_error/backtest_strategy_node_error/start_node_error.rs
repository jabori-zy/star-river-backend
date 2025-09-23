use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StartNodeError {
    #[snafu(display("start node backtest config field [{field_name}]'s value is null"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("start node backtest config deserialization failed. reason: [{source}]"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    // >= 0
    #[snafu(display(
        "start node config [{config_name}] should be greater than or equal to(>= 0) zero, but got [{config_value}]"
    ))]
    ValueNotGreaterThanOrEqualToZero {
        node_name: String,
        node_id: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    // > 0
    #[snafu(display(
        "start node [{node_name}({node_id})] config [{config_name}] should be greater than(> 0) zero, but got [{config_value}]"
    ))]
    ValueNotGreaterThanZero {
        node_name: String,
        node_id: String,
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for StartNodeError {
    fn get_prefix(&self) -> &'static str {
        "START_NODE"
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
        matches!(
            self,
            StartNodeError::ConfigFieldValueNull { .. }
                | StartNodeError::ConfigDeserializationFailed { .. }
                | StartNodeError::ValueNotGreaterThanOrEqualToZero { .. }
                | StartNodeError::ValueNotGreaterThanZero { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => {
                // 直接使用 Display trait 中定义的英文消息
                self.to_string()
            }
            Language::Chinese => match self {
                StartNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("开始节点回测配置字段 [{}] 的值为空", field_name)
                }
                StartNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("开始节点回测配置反序列化失败，原因: [{}]", source)
                }
                StartNodeError::ValueNotGreaterThanOrEqualToZero {
                    node_name,
                    node_id,
                    config_name,
                    config_value,
                    ..
                } => {
                    format!(
                        "开始节点 [{}({})] 配置 [{}] 应该大于等于零(>= 0)，但值为 [{}]",
                        node_name, node_id, config_name, config_value
                    )
                }
                StartNodeError::ValueNotGreaterThanZero {
                    node_name,
                    node_id,
                    config_name,
                    config_value,
                    ..
                } => {
                    format!(
                        "开始节点 [{}({})] 配置 [{}] 应该大于零(> 0)，但值为 [{}]",
                        node_name, node_id, config_name, config_value
                    )
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // ConfigDeserializationFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            StartNodeError::ConfigDeserializationFailed { .. } => vec![self.error_code()],

            // Other errors have no source - return own error code
            _ => vec![self.error_code()],
        }
    }
}
