use sea_orm::DbErr;
use snafu::{Backtrace, Snafu};

use crate::error::{
    ErrorCode,
    error_trait::{ErrorLanguage, StarRiverErrorTrait},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StarRiverError {
    #[snafu(display("update system config failed. reason: [{source}]"))]
    UpdateSystemConfigFailed { source: DbErr, backtrace: Backtrace },

    #[snafu(display("get system config failed. reason: [{source}]"))]
    GetSystemConfigFailed { source: DbErr, backtrace: Backtrace },

    #[snafu(display("invalid key type: {key_type}"))]
    InvalidKeyType { key_type: String, backtrace: Backtrace },

    #[snafu(display("invalid indicator type: {indicator_type}"))]
    InvalidIndicatorType { indicator_type: String, backtrace: Backtrace },

    #[snafu(display("invalid key format: {key_str}"))]
    InvalidKeyFormat { key_str: String, backtrace: Backtrace },

    #[snafu(display("parse exchange failed: {exchange}"))]
    ParseExchangeFailed { exchange: String, backtrace: Backtrace },

    #[snafu(display("parse kline interval failed: {interval}, reason: [{source}]"))]
    ParseKlineIntervalFailed {
        interval: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse datetime failed: {datetime}. reason: {source}"))]
    ParseDataTimeFailed {
        datetime: String,
        source: chrono::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid timestamp: {timestamp}"))]
    InvalidTimestamp { timestamp: i64, backtrace: Backtrace },

    #[snafu(display("Transform timestamp failed: {timestamp}"))]
    TransformTimestampFailed { timestamp: i64, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for StarRiverError {
    fn get_prefix(&self) -> &'static str {
        "StarRiver"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StarRiverError::UpdateSystemConfigFailed { .. } => 1001, // Update system config failed
            StarRiverError::GetSystemConfigFailed { .. } => 1002,    // Get system config failed
            StarRiverError::InvalidKeyType { .. } => 1003,           // Invalid cache key type
            StarRiverError::InvalidIndicatorType { .. } => 1004,     // Invalid indicator type
            StarRiverError::InvalidKeyFormat { .. } => 1005,         // Invalid cache key format
            StarRiverError::ParseExchangeFailed { .. } => 1006,      // Parse exchange failed
            StarRiverError::ParseKlineIntervalFailed { .. } => 1007, // Parse kline interval failed
            StarRiverError::ParseDataTimeFailed { .. } => 1008,      // Parse datetime failed
            StarRiverError::InvalidTimestamp { .. } => 1009,         // Invalid timestamp
            StarRiverError::TransformTimestampFailed { .. } => 1010, // Transform timestamp failed
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                StarRiverError::UpdateSystemConfigFailed { source, .. } => {
                    format!("更新系统配置失败，原因: {}", source)
                }
                StarRiverError::GetSystemConfigFailed { source, .. } => {
                    format!("获取系统配置失败，原因: {}", source)
                }
                StarRiverError::InvalidKeyType { key_type, .. } => {
                    format!("无效的缓存键类型: {}", key_type)
                }
                StarRiverError::InvalidIndicatorType { indicator_type, .. } => {
                    format!("无效的指标类型: {}", indicator_type)
                }
                StarRiverError::InvalidKeyFormat { key_str, .. } => {
                    format!("无效的缓存键格式: {}", key_str)
                }
                StarRiverError::ParseExchangeFailed { exchange, .. } => {
                    format!("解析交易所失败: {}", exchange)
                }
                StarRiverError::ParseKlineIntervalFailed { interval, source, .. } => {
                    format!("解析K线周期失败: {}. 原因: {}", interval, source)
                }
                StarRiverError::ParseDataTimeFailed { datetime, source, .. } => {
                    format!("解析时间失败: {}. 原因: {}", datetime, source)
                }
                StarRiverError::InvalidTimestamp { timestamp, .. } => {
                    format!("非法时间戳: {}", timestamp)
                }
                StarRiverError::TransformTimestampFailed { timestamp, .. } => {
                    format!("时间戳转换失败: {}", timestamp)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            StarRiverError::UpdateSystemConfigFailed { .. }
            | StarRiverError::GetSystemConfigFailed { .. }
            | StarRiverError::InvalidKeyType { .. }
            | StarRiverError::InvalidIndicatorType { .. }
            | StarRiverError::InvalidKeyFormat { .. }
            | StarRiverError::ParseExchangeFailed { .. }
            | StarRiverError::ParseKlineIntervalFailed { .. }
            | StarRiverError::ParseDataTimeFailed { .. }
            | StarRiverError::InvalidTimestamp { .. }
            | StarRiverError::TransformTimestampFailed { .. } => vec![self.error_code()],
        }
    }
}
