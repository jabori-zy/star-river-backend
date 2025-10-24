use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum VariableNodeError {
    #[snafu(display("get variable node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("get variable node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed { source: serde_json::Error, backtrace: Backtrace },


    #[snafu(display("the symbol of system variable is null: {sys_var_name}"))]
    SysVariableSymbolIsNull {
        sys_var_name: String,
    }
}

// Implement the StarRiverErrorTrait for GetVariableNodeError
impl crate::error::error_trait::StarRiverErrorTrait for VariableNodeError {
    fn get_prefix(&self) -> &'static str {
        "VARIABLE_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            VariableNodeError::ConfigFieldValueNull { .. } => 1001, //Config字段值为空
            VariableNodeError::ConfigDeserializationFailed { .. } => 1002, //Config反序列化失败
            VariableNodeError::SysVariableSymbolIsNull { .. } => 1003, //系统变量交易对为空
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
            VariableNodeError::ConfigFieldValueNull { .. } | VariableNodeError::ConfigDeserializationFailed { .. }
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
                VariableNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("获取变量节点回测配置字段值为空: {}", field_name)
                }
                VariableNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("获取变量节点回测配置反序列化失败，原因: {}", source)
                }
                VariableNodeError::SysVariableSymbolIsNull { sys_var_name, .. } => {
                    format!("系统变量 [{}] 的交易对为空", sys_var_name)
                }
            },
        }
    }
}
