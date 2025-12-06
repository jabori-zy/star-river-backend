use axum::http::StatusCode;
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum TaLibError {
    #[snafu(display("indicator type [{indicator_type}] is unsupported"))]
    UnsupportType { indicator_type: String, backtrace: Backtrace },

    #[snafu(display("create indicator [{indicator_type}] failed. reason: [{source}]"))]
    CreateIndicatorFailed {
        indicator_type: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "invalid indicator config format: {indicator_config}. the format should be 'indicator_name(param1=value1 param2=value2 ...)'"
    ))]
    InvalidConfigFormat { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator param empty: {indicator_config}"))]
    ParamEmpty { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator param format invalid: {indicator_config}. the format should be 'key=value'"))]
    ParamFormatInvalid { indicator_config: String, backtrace: Backtrace },

    #[snafu(display("indicator config miss param {param}"))]
    ConfigMissParam { param: String, backtrace: Backtrace },

    #[snafu(display("parse int indicator param failed: {param}. reason: [{source}]"))]
    ParseIntParamFailed {
        param: String,
        source: std::num::ParseIntError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse float indicator param failed: {param}. reason: [{source}]"))]
    ParseFloatParamFailed {
        param: String,
        source: std::num::ParseFloatError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse special indicator param failed: {param}. reason: {reason}"))]
    ParseSpecialParamFailed {
        param: String,
        reason: String,
        backtrace: Backtrace,
    },

    #[snafu(display("{indicator_name} lookback is {lookback} but data length is {data_length}"))]
    DataLessThenLookback {
        indicator_name: String,
        lookback: usize,
        data_length: usize,
        backtrace: Backtrace,
    },

    #[snafu(display("data length not equal: {data_length:?}"))]
    DataLengthNotEqual { data_length: Vec<usize>, backtrace: Backtrace },

    #[snafu(display("TA-Lib error code: {ret_code}"))]
    TalibErrorCode {
        #[cfg(target_os = "windows")]
        ret_code: i32,
        #[cfg(target_os = "macos")]
        ret_code: u32,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl TaLibError {
    pub fn get_prefix(&self) -> &'static str {
        "TA_LIB"
    }

    pub fn error_code(&self) -> String {
        let prefix = self.get_prefix();
        let code = match self {
            TaLibError::UnsupportType { .. } => 1001,
            TaLibError::CreateIndicatorFailed { .. } => 1002,
            TaLibError::InvalidConfigFormat { .. } => 1003,
            TaLibError::ParamEmpty { .. } => 1004,
            TaLibError::ParamFormatInvalid { .. } => 1005,
            TaLibError::ConfigMissParam { .. } => 1006,
            TaLibError::ParseIntParamFailed { .. } => 1007,
            TaLibError::ParseFloatParamFailed { .. } => 1008,
            TaLibError::ParseSpecialParamFailed { .. } => 1009,
            TaLibError::DataLessThenLookback { .. } => 1010,
            TaLibError::DataLengthNotEqual { .. } => 1011,
            TaLibError::TalibErrorCode { .. } => 1012,
        };
        format!("{}_{:04}", prefix, code)
    }

    pub fn http_status_code(&self) -> StatusCode {
        match self {
            // Client errors - BAD_REQUEST (400)
            TaLibError::UnsupportType { .. } => StatusCode::BAD_REQUEST,
            TaLibError::InvalidConfigFormat { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ParamEmpty { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ParamFormatInvalid { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ConfigMissParam { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ParseIntParamFailed { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ParseFloatParamFailed { .. } => StatusCode::BAD_REQUEST,
            TaLibError::ParseSpecialParamFailed { .. } => StatusCode::BAD_REQUEST,
            TaLibError::DataLessThenLookback { .. } => StatusCode::BAD_REQUEST,
            TaLibError::DataLengthNotEqual { .. } => StatusCode::BAD_REQUEST,

            // Server errors - INTERNAL_SERVER_ERROR (500)
            TaLibError::CreateIndicatorFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            TaLibError::TalibErrorCode { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_message(&self, language: &str) -> String {
        match language {
            "english" => self.to_string(),
            "chinese" => match self {
                TaLibError::UnsupportType { indicator_type, .. } => {
                    format!("不支持的指标类型 [{}]", indicator_type)
                }
                TaLibError::CreateIndicatorFailed {
                    indicator_type, source, ..
                } => {
                    format!("创建指标 [{}] 失败，原因: [{}]", indicator_type, source)
                }
                TaLibError::InvalidConfigFormat { indicator_config, .. } => {
                    format!("无效的指标配置格式: {}", indicator_config)
                }
                TaLibError::ParamEmpty { indicator_config, .. } => {
                    format!("指标参数为空: {}", indicator_config)
                }
                TaLibError::ParamFormatInvalid { indicator_config, .. } => {
                    format!("指标参数格式无效: {}", indicator_config)
                }
                TaLibError::ConfigMissParam { param, .. } => {
                    format!("指标配置缺少参数: {}", param)
                }
                TaLibError::ParseIntParamFailed { param, source, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, source)
                }
                TaLibError::ParseFloatParamFailed { param, source, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, source)
                }
                TaLibError::ParseSpecialParamFailed { param, reason, .. } => {
                    format!("指标参数解析失败: {}. 原因: {}", param, reason)
                }
                TaLibError::DataLessThenLookback {
                    indicator_name,
                    lookback,
                    data_length,
                    ..
                } => {
                    format!("{} 的 lookback 是 {} 但数据长度是 {}", indicator_name, lookback, data_length)
                }
                TaLibError::DataLengthNotEqual { data_length, .. } => {
                    format!(
                        "数据长度不一致: {}",
                        data_length.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")
                    )
                }
                TaLibError::TalibErrorCode { ret_code, .. } => {
                    format!("TA-Lib 错误代码: {}", ret_code)
                }
            },
            _ => "".to_string(),
        }
    }

    pub fn error_code_chain(&self) -> Vec<String> {
        match self {
            TaLibError::UnsupportType { .. }
            | TaLibError::CreateIndicatorFailed { .. }
            | TaLibError::InvalidConfigFormat { .. }
            | TaLibError::ParamEmpty { .. }
            | TaLibError::ParamFormatInvalid { .. }
            | TaLibError::ConfigMissParam { .. }
            | TaLibError::ParseIntParamFailed { .. }
            | TaLibError::ParseFloatParamFailed { .. }
            | TaLibError::ParseSpecialParamFailed { .. }
            | TaLibError::DataLessThenLookback { .. }
            | TaLibError::DataLengthNotEqual { .. }
            | TaLibError::TalibErrorCode { .. } => vec![self.error_code()],
        }
    }
}
