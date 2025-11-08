use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use exchange_core::error::state_machine_error::ExchangeStateMachineError;
use crate::binance::data_processor_error::BinanceDataProcessorError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BinanceError {
    #[snafu(transparent)]
    DataProcessorError { source: BinanceDataProcessorError, backtrace: Backtrace },

    #[snafu(transparent)]
    StateMachineError { source: ExchangeStateMachineError, backtrace: Backtrace },

    #[snafu(display("http client not created"))]
    HttpClientNotCreated { backtrace: Backtrace },

    

    #[snafu(display("network error: url: {url}, source: {source}"))]
    Network {
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("ping failed"))]
    PingFailed { backtrace: Backtrace },

    #[snafu(display("response error: url: {url}, source: {source}"))]
    Response {
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("parse server time failed: {source}"))]
    ParseServerTimeFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("parse raw data {data_name} failed: {source}"))]
    ParseRawDataFailed {
        data_name: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("parse date time {timestamp} failed"))]
    DateTimeParseFailed { timestamp: i64, backtrace: Backtrace },

    TypeConversionFailed {
        from: String,
        to: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("parse number failed: field: {field}, value: {value}, source: {source}"))]
    ParseNumberFailed {
        field: String,
        value: String,
        source: std::num::ParseFloatError,
        backtrace: Backtrace,
    },

    #[snafu(display("missing field: {field}"))]
    MissingField { field: String, backtrace: Backtrace },

    #[snafu(display("invalid field type: {field}, expected: {expected}"))]
    InvalidFieldType {
        field: String,
        expected: String,
        backtrace: Backtrace,
    },

    #[snafu(display("symbol {symbol} not found"))]
    SymbolNotFound { symbol: String, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl StarRiverErrorTrait for BinanceError {
    fn get_prefix(&self) -> &'static str {
        "BINANCE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            
            BinanceError::StateMachineError { .. } => 1001,     // 状态机错误
            BinanceError::DataProcessorError { .. } => 1002,     // 数据处理器错误
            BinanceError::HttpClientNotCreated { .. } => 1003,  // 客户端未创建
            BinanceError::PingFailed { .. } => 1004,            // Ping失败
            BinanceError::Network { .. } => 1005,               // 网络错误
            BinanceError::Response { .. } => 1006,              // 响应错误
            BinanceError::ParseServerTimeFailed { .. } => 1007, // 解析服务器时间失败
            BinanceError::ParseRawDataFailed { .. } => 1008,    // 解析原始数据失败
            BinanceError::DateTimeParseFailed { .. } => 1009,   // 解析时间戳失败
            BinanceError::TypeConversionFailed { .. } => 1010,  // 类型转换失败
            BinanceError::ParseNumberFailed { .. } => 1011,     // 解析数字失败
            BinanceError::MissingField { .. } => 1012,          // 缺少字段
            BinanceError::InvalidFieldType { .. } => 1013,      // 字段类型无效
            BinanceError::SymbolNotFound { .. } => 1014,        // 交易对不存在
        };
        format!("{}_{:04}", prefix, code)
    }


    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BinanceError::StateMachineError { source, .. } => source.error_message(language),
                BinanceError::DataProcessorError { source, .. } => source.error_message(language),
                BinanceError::HttpClientNotCreated { .. } => {
                    format!("客户端未创建")
                }
                BinanceError::PingFailed { .. } => {
                    format!("Ping失败")
                }
                BinanceError::Network { url, source, .. } => {
                    format!("网络错误: url: {}, source: {}", url, source)
                }
                BinanceError::Response { url, source, .. } => {
                    format!("响应错误: url: {}, source: {}", url, source)
                }
                BinanceError::ParseServerTimeFailed { source, .. } => {
                    format!("解析服务器时间失败: {}", source)
                }
                BinanceError::ParseRawDataFailed { data_name, source, .. } => {
                    format!("解析原始数据{data_name}失败: {}", source)
                }
                BinanceError::DateTimeParseFailed { timestamp, .. } => {
                    format!("解析时间戳{timestamp}失败.")
                }
                BinanceError::TypeConversionFailed { from, to, source, .. } => {
                    format!("类型转换失败: from: {}, to: {}, source: {}", from, to, source)
                }
                BinanceError::ParseNumberFailed { field, value, source, .. } => {
                    format!("解析数字失败: 字段: {}, 值: {}, 错误: {}", field, value, source)
                }
                BinanceError::MissingField { field, .. } => {
                    format!("缺少字段: {}", field)
                }
                BinanceError::InvalidFieldType { field, expected, .. } => {
                    format!("字段类型无效: 字段: {}, 期望: {}", field, expected)
                }
                BinanceError::SymbolNotFound { symbol, .. } => {
                    format!("交易对 {} 不存在", symbol)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BinanceError::StateMachineError { source, .. } => generate_error_code_chain(source),
            BinanceError::DataProcessorError { source, .. } => generate_error_code_chain(source),
            // CreateIndicatorFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            BinanceError::HttpClientNotCreated { .. }
            | BinanceError::PingFailed { .. }
            | BinanceError::Network { .. }
            | BinanceError::Response { .. }
            | BinanceError::ParseServerTimeFailed { .. }
            | BinanceError::ParseRawDataFailed { .. }
            | BinanceError::DateTimeParseFailed { .. }
            | BinanceError::TypeConversionFailed { .. }
            | BinanceError::ParseNumberFailed { .. }
            | BinanceError::MissingField { .. }
            | BinanceError::InvalidFieldType { .. }
            | BinanceError::SymbolNotFound { .. } => vec![self.error_code()],
        }
    }
}
