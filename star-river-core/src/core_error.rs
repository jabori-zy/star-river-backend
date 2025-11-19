use sea_orm::DbErr;
use snafu::{Backtrace, Snafu};

use crate::error::{
    ErrorCode,
    error_trait::{ErrorLanguage, StarRiverErrorTrait},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CoreError {
    #[snafu(display("update system config failed. reason: [{source}]"))]
    UpdateSystemConfigFailed { source: DbErr, backtrace: Backtrace },

    #[snafu(display("get system config failed. reason: [{source}]"))]
    GetSystemConfigFailed { source: DbErr, backtrace: Backtrace },


    #[snafu(display("parse exchange failed: {exchange}"))]
    ParseExchangeFailed { exchange: String, backtrace: Backtrace },



}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for CoreError {
    fn get_prefix(&self) -> &'static str {
        "Core"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            CoreError::UpdateSystemConfigFailed { .. } => 1001, // 更新系统配置失败
            CoreError::GetSystemConfigFailed { .. } => 1002,    // 获取系统配置失败
            CoreError::ParseExchangeFailed { .. } => 1003,      // 解析交易所失败
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                CoreError::UpdateSystemConfigFailed { source, .. } => {
                    format!("更新系统配置失败，原因: {}", source)
                }
                CoreError::GetSystemConfigFailed { source, .. } => {
                    format!("获取系统配置失败，原因: {}", source)
                }
                CoreError::ParseExchangeFailed { exchange, .. } => {
                    format!("解析交易所失败: {}", exchange)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            CoreError::UpdateSystemConfigFailed { .. }
            | CoreError::GetSystemConfigFailed { .. }
            | CoreError::ParseExchangeFailed { .. } => vec![self.error_code()],
        }
    }
}
