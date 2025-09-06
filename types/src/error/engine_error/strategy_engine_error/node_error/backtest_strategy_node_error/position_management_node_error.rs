use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use crate::error::error_trait::Language;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum PositionManagementNodeError {

    #[snafu(display("position management node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull {
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("position management node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    }
}

// Implement the StarRiverErrorTrait for PositionManagementNodeError
impl crate::error::error_trait::StarRiverErrorTrait for PositionManagementNodeError {
    fn get_prefix(&self) -> &'static str {
        "POSITION_MANAGEMENT_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            PositionManagementNodeError::ConfigFieldValueNull { .. } => 1001,
            PositionManagementNodeError::ConfigDeserializationFailed { .. } => 1002,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            PositionManagementNodeError::ConfigFieldValueNull { .. } |
            PositionManagementNodeError::ConfigDeserializationFailed { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All PositionManagementNodeError variants either have no source or
        // have external sources (serde_json::Error) that don't implement our trait
        vec![self.error_code()]
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => {
                self.to_string()
            },
            Language::Chinese => {
                match self {
                    PositionManagementNodeError::ConfigFieldValueNull { field_name, .. } => {
                        format!("仓位管理节点回测配置字段值为空: {}", field_name)
                    },
                    PositionManagementNodeError::ConfigDeserializationFailed { source, .. } => {
                        format!("仓位管理节点回测配置反序列化失败，原因: {}", source)
                    },
                }
            },
        }
    }
}