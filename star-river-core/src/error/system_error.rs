use crate::error::error_trait::Language;
use crate::error::ErrorCode;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use sea_orm::DbErr;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum SystemError {
    #[snafu(display("update system config failed. reason: [{source}]"))]
    UpdateSystemConfigFailed {
        source: DbErr,
        backtrace: Backtrace,
    },

    #[snafu(display("get system config failed. reason: [{source}]"))]
    GetSystemConfigFailed {
        source: DbErr,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for SystemError
impl crate::error::error_trait::StarRiverErrorTrait for SystemError {
    fn get_prefix(&self) -> &'static str {
        "SYSTEM"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            SystemError::UpdateSystemConfigFailed { .. } => 1001,
            SystemError::GetSystemConfigFailed { .. } => 1002,
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
            SystemError::UpdateSystemConfigFailed { .. } |
            SystemError::GetSystemConfigFailed { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                SystemError::UpdateSystemConfigFailed { source, .. } => {
                    format!("更新系统配置失败，原因: [{}]", source)
                }
                SystemError::GetSystemConfigFailed { source, .. } => {
                    format!("获取系统配置失败，原因: [{}]", source)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            SystemError::UpdateSystemConfigFailed { .. } |
            SystemError::GetSystemConfigFailed { .. } => vec![self.error_code()],
        }
    }
}
