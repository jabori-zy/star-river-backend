use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StartNodeError {
    // >= 0
    #[snafu(display("start node config [{config_name}] should be greater than or equal to(>= 0) zero, but got [{config_value}]"))]
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
            StartNodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1001,
            StartNodeError::ValueNotGreaterThanZero { .. } => 1002,
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
            StartNodeError::ValueNotGreaterThanOrEqualToZero { .. }
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
        vec![self.error_code()]
    }
}
