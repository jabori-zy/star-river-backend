use snafu::{Backtrace, Snafu};
use star_river_core::{
    core_error::CoreError,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum KeyError {
    #[snafu(transparent)]
    CoreError { source: CoreError, backtrace: Backtrace },

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
}

// Implement the StarRiverErrorTrait for KeyError
impl StarRiverErrorTrait for KeyError {
    fn get_prefix(&self) -> &'static str {
        "Key"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            KeyError::CoreError { .. } => 1000,                // 核心错误
            KeyError::InvalidKeyType { .. } => 1001,           // 无效的缓存键类型
            KeyError::InvalidIndicatorType { .. } => 1002,     // 无效的指标类型
            KeyError::InvalidKeyFormat { .. } => 1003,         // 无效的缓存键格式
            KeyError::ParseExchangeFailed { .. } => 1004,      // 解析交易所失败
            KeyError::ParseKlineIntervalFailed { .. } => 1005, // 解析K线周期失败
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                KeyError::CoreError { source, .. } => source.error_message(language),
                KeyError::InvalidKeyType { key_type, .. } => {
                    format!("无效的缓存键类型: {}", key_type)
                }
                KeyError::InvalidIndicatorType { indicator_type, .. } => {
                    format!("无效的指标类型: {}", indicator_type)
                }
                KeyError::InvalidKeyFormat { key_str, .. } => {
                    format!("无效的缓存键格式: {}", key_str)
                }
                KeyError::ParseExchangeFailed { exchange, .. } => {
                    format!("解析交易所失败: {}", exchange)
                }
                KeyError::ParseKlineIntervalFailed { interval, .. } => {
                    format!("解析K线周期失败: {}", interval)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            KeyError::CoreError { source, .. } => generate_error_code_chain(source),
            KeyError::InvalidKeyType { .. }
            | KeyError::InvalidIndicatorType { .. }
            | KeyError::InvalidKeyFormat { .. }
            | KeyError::ParseExchangeFailed { .. }
            | KeyError::ParseKlineIntervalFailed { .. } => vec![self.error_code()],
        }
    }
}
