use crate::error::ErrorCode;
use crate::error::error_trait::ErrorLanguage;
use crate::error::error_trait::StarRiverErrorTrait;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorEngineError {
    #[snafu(display("indicator [{indicator_name}] data length {data_length} is less than lookback {lookback}"))]
    DataLessThenLookback {
        indicator_name: String,
        lookback: usize,
        data_length: usize,
        backtrace: Backtrace,
    },

    #[snafu(display("invalid kline data at index {index}"))]
    InvalidKlineData { index: usize, backtrace: Backtrace },

    #[snafu(display("TA-Lib error code: {ret_code}"))]
    Talib {
        #[cfg(target_os = "windows")]
        ret_code: i32,
        #[cfg(target_os = "macos")]
        ret_code: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("data length not equal: {data_length:?}"))]
    DataLengthNotEqual { data_length: Vec<usize>, backtrace: Backtrace },
}

impl StarRiverErrorTrait for IndicatorEngineError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IndicatorEngineError::DataLessThenLookback { .. } => 1001, // 数据长度小于lookback
            IndicatorEngineError::InvalidKlineData { .. } => 1002,     // 无效的k线数据
            IndicatorEngineError::Talib { .. } => 1003,                // TA-Lib错误
            IndicatorEngineError::DataLengthNotEqual { .. } => 1004,   // 数据长度不等于
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> std::collections::HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self, IndicatorEngineError::DataLessThenLookback { .. })
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            IndicatorEngineError::DataLessThenLookback { .. } => vec![self.error_code()],
            IndicatorEngineError::InvalidKlineData { .. } => vec![self.error_code()],
            IndicatorEngineError::Talib { .. } => vec![self.error_code()],
            IndicatorEngineError::DataLengthNotEqual { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                IndicatorEngineError::DataLessThenLookback {
                    indicator_name,
                    lookback,
                    data_length,
                    ..
                } => {
                    format!("指标 [{}] 数据长度 {} 小于 lookback {}", indicator_name, data_length, lookback)
                }
                IndicatorEngineError::InvalidKlineData { index, .. } => {
                    format!("在index为{}的位置是无效的k线数据", index)
                }
                IndicatorEngineError::Talib { ret_code, .. } => {
                    format!("TA-Lib错误代码: {}", ret_code)
                }
                IndicatorEngineError::DataLengthNotEqual { data_length, .. } => {
                    format!(
                        "数据长度不一致: {}",
                        data_length.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")
                    )
                }
            },
        }
    }
}
