use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::error_trait::StarRiverErrorTrait;
use sea_orm::DbErr;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

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

    #[snafu(display(
        "invalid indicator config format: {indicator_config}. the format should be 'indicator_name(param1=value1 param2=value2 ...)"
    ))]
    InvalidIndicatorConfigFormat { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator param empty: {indicator_config}"))]
    IndicatorParamEmpty { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator param format invalid: {indicator_config}. the format should be 'key=value'"))]
    IndicatorParamFormatInvalid { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator config miss param {param}"))]
    IndicatorConfigMissParam { param: String, backtrace: Backtrace },

    #[snafu(display("parse int indicator param failed: {param}. reason: [{source}]"))]
    ParseIntIndicatorParamFailed {
        param: String,
        source: std::num::ParseIntError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse float indicator param failed: {param}. reason: [{source}]"))]
    ParseFloatIndicatorParamFailed {
        param: String,
        source: std::num::ParseFloatError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse special indicator param failed: {param}. reason: {reason}"))]
    ParseSpecialIndicatorParamFailed {
        param: String,
        reason: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for StarRiverError {
    fn get_prefix(&self) -> &'static str {
        "StarRiver"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StarRiverError::UpdateSystemConfigFailed { .. } => 1001,         // 更新系统配置失败
            StarRiverError::GetSystemConfigFailed { .. } => 1002,            // 获取系统配置失败
            StarRiverError::InvalidKeyType { .. } => 1003,                   // 无效的缓存键类型
            StarRiverError::InvalidIndicatorType { .. } => 1004,             // 无效的指标类型
            StarRiverError::InvalidIndicatorConfigFormat { .. } => 1005,     // 无效的指标配置格式
            StarRiverError::IndicatorParamEmpty { .. } => 1006,              // 指标参数为空
            StarRiverError::IndicatorParamFormatInvalid { .. } => 1007,      // 指标参数格式无效
            StarRiverError::IndicatorConfigMissParam { .. } => 1008,         // 指标配置缺少参数
            StarRiverError::ParseIntIndicatorParamFailed { .. } => 1009,     // 指标参数解析失败
            StarRiverError::ParseFloatIndicatorParamFailed { .. } => 1010,   // 指标参数解析失败
            StarRiverError::ParseSpecialIndicatorParamFailed { .. } => 1011, // 指标参数解析失败
            StarRiverError::InvalidKeyFormat { .. } => 1012,                 // 无效的缓存键格式
            StarRiverError::ParseExchangeFailed { .. } => 1013,              // 解析交易所失败
            StarRiverError::ParseKlineIntervalFailed { .. } => 1014,         // 解析K线周期失败
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
            StarRiverError::UpdateSystemConfigFailed { .. }
                | StarRiverError::GetSystemConfigFailed { .. }
                | StarRiverError::InvalidKeyType { .. }
                | StarRiverError::InvalidIndicatorType { .. }
                | StarRiverError::InvalidIndicatorConfigFormat { .. }
                | StarRiverError::IndicatorParamEmpty { .. }
                | StarRiverError::IndicatorParamFormatInvalid { .. }
                | StarRiverError::IndicatorConfigMissParam { .. }
                | StarRiverError::ParseIntIndicatorParamFailed { .. }
                | StarRiverError::ParseFloatIndicatorParamFailed { .. }
                | StarRiverError::ParseSpecialIndicatorParamFailed { .. }
                | StarRiverError::InvalidKeyFormat { .. }
                | StarRiverError::ParseExchangeFailed { .. }
                | StarRiverError::ParseKlineIntervalFailed { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
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
                StarRiverError::InvalidIndicatorConfigFormat { indicator_config, .. } => {
                    format!("无效的指标配置格式: {}", indicator_config)
                }
                StarRiverError::IndicatorParamEmpty { indicator_config, .. } => {
                    format!("指标参数为空: {}", indicator_config)
                }
                StarRiverError::IndicatorParamFormatInvalid { indicator_config, .. } => {
                    format!("指标参数格式无效: {}", indicator_config)
                }
                StarRiverError::IndicatorConfigMissParam { param, .. } => {
                    format!("指标配置缺少参数: {}", param)
                }
                StarRiverError::ParseIntIndicatorParamFailed { param, source, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, source)
                }
                StarRiverError::ParseFloatIndicatorParamFailed { param, source, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, source)
                }
                StarRiverError::ParseSpecialIndicatorParamFailed { param, reason, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, reason)
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
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            StarRiverError::UpdateSystemConfigFailed { .. }
            | StarRiverError::GetSystemConfigFailed { .. }
            | StarRiverError::InvalidKeyType { .. }
            | StarRiverError::InvalidIndicatorType { .. }
            | StarRiverError::InvalidIndicatorConfigFormat { .. }
            | StarRiverError::IndicatorParamEmpty { .. }
            | StarRiverError::IndicatorParamFormatInvalid { .. }
            | StarRiverError::IndicatorConfigMissParam { .. }
            | StarRiverError::ParseIntIndicatorParamFailed { .. }
            | StarRiverError::ParseFloatIndicatorParamFailed { .. }
            | StarRiverError::ParseSpecialIndicatorParamFailed { .. }
            | StarRiverError::InvalidKeyFormat { .. }
            | StarRiverError::ParseExchangeFailed { .. }
            | StarRiverError::ParseKlineIntervalFailed { .. } => vec![self.error_code()],
        }
    }
}
