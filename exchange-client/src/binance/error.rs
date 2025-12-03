use exchange_core::{KlineInterval, error::state_machine_error::ExchangeStateMachineError};
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

use crate::binance::data_processor_error::BinanceDataProcessorError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BinanceError {
    #[snafu(transparent)]
    DataProcessorError {
        source: BinanceDataProcessorError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    StateMachineError {
        source: ExchangeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("http client not created"))]
    HttpClientNotCreated { backtrace: Backtrace },

    #[snafu(display("websocket connection failed: url: {url}, source: {source}"))]
    WebSocketConnectionFailed {
        url: String,
        source: TungsteniteError,
        backtrace: Backtrace,
    },

    #[snafu(display("Network error: url: {url}"))]
    Network {
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("ping failed"))]
    PingFailed { backtrace: Backtrace },

    #[snafu(display("Response error: url: {url}"))]
    Response {
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("parse server time failed"))]
    ParseServerTimeFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("parse raw data {data_name} failed"))]
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

    #[snafu(display("binance unsupported kline interval: {interval}"))]
    UnsupportedKlineInterval { interval: KlineInterval, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl StarRiverErrorTrait for BinanceError {
    fn get_prefix(&self) -> &'static str {
        "BINANCE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            BinanceError::StateMachineError { .. } => 1001,         // 状态机错误
            BinanceError::DataProcessorError { .. } => 1002,        // 数据处理器错误
            BinanceError::HttpClientNotCreated { .. } => 1003,      // 客户端未创建
            BinanceError::WebSocketConnectionFailed { .. } => 1004, // WebSocket连接失败
            BinanceError::PingFailed { .. } => 1005,                // Ping失败
            BinanceError::Network { .. } => 1006,                   // 网络错误
            BinanceError::Response { .. } => 1007,                  // 响应错误
            BinanceError::ParseServerTimeFailed { .. } => 1008,     // 解析服务器时间失败
            BinanceError::ParseRawDataFailed { .. } => 1009,        // 解析原始数据失败
            BinanceError::DateTimeParseFailed { .. } => 1010,       // 解析时间戳失败
            BinanceError::TypeConversionFailed { .. } => 1011,      // 类型转换失败
            BinanceError::ParseNumberFailed { .. } => 1012,         // 解析数字失败
            BinanceError::MissingField { .. } => 1013,              // 缺少字段
            BinanceError::InvalidFieldType { .. } => 1014,          // 字段类型无效
            BinanceError::SymbolNotFound { .. } => 1015,            // 交易对不存在
            BinanceError::UnsupportedKlineInterval { .. } => 1016,  // 不支持的K线周期
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BinanceError::StateMachineError { source, .. } => source.error_message(language),
                BinanceError::DataProcessorError { source, .. } => source.error_message(language),
                BinanceError::WebSocketConnectionFailed { url, source, .. } => {
                    format!("WebSocket连接失败: url: {}, source: {}", url, source)
                }
                BinanceError::HttpClientNotCreated { .. } => {
                    format!("客户端未创建")
                }
                BinanceError::PingFailed { .. } => {
                    format!("Ping失败")
                }
                BinanceError::Network { url, .. } => {
                    format!("网络错误: url: {}", url)
                }
                BinanceError::Response { url, .. } => {
                    format!("响应错误: url: {}", url)
                }
                BinanceError::ParseServerTimeFailed { .. } => {
                    format!("解析服务器时间失败.")
                }
                BinanceError::ParseRawDataFailed { data_name, .. } => {
                    format!("解析原始数据{data_name}失败.")
                }
                BinanceError::DateTimeParseFailed { timestamp, .. } => {
                    format!("解析时间戳{timestamp}失败.")
                }
                BinanceError::TypeConversionFailed { from, to, .. } => {
                    format!("类型转换失败: from: {}, to: {}", from, to)
                }
                BinanceError::ParseNumberFailed { field, value, .. } => {
                    format!("解析数字失败: 字段: {}, 值: {}", field, value)
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
                BinanceError::UnsupportedKlineInterval { interval, .. } => {
                    format!("币安不支持的K线周期: {}", interval)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BinanceError::StateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            BinanceError::DataProcessorError { source, .. } => generate_error_code_chain(source, self.error_code()),
            // CreateIndicatorFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            BinanceError::WebSocketConnectionFailed { .. }
            | BinanceError::HttpClientNotCreated { .. }
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
            | BinanceError::SymbolNotFound { .. }
            | BinanceError::UnsupportedKlineInterval { .. } => vec![self.error_code()],
        }
    }
}
