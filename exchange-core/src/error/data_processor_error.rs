use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait};

/// Generic data processor error
///
/// This error type is used for all exchange data processing errors.
/// Different exchanges are distinguished by different prefixes (e.g., BINANCE_DATA_PROCESSOR, MT5_DATA_PROCESSOR)
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DataProcessorError {
    // #[snafu(transparent)]
    // StarRiverError { source: StarRiverError, backtrace: Backtrace },

    #[snafu(display("JSON parsing failed"))]
    JsonParseFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("Array data parsing failed: expected array type, got {actual_type}"))]
    ArrayParseFailed { actual_type: String, backtrace: Backtrace },

    #[snafu(display("Enum parsing failed for field '{field}': unknown variant '{variant}'"))]
    EnumParseFailed {
        field: String,
        variant: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Required field '{field}' missing in JSON data"))]
    MissingField {
        field: String,
        context: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Value is None for field '{field}'"))]
    ValueIsNone { field: String, backtrace: Backtrace },

    #[snafu(display("Invalid data type for field '{field}': expected {expected}, got {actual}"))]
    InvalidFieldType {
        field: String,
        expected: String,
        actual: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid kline array format: expected 6 elements [timestamp, open, high, low, close, volume], got {length}"))]
    InvalidKlineArrayFormat { length: usize, data: String, backtrace: Backtrace },

    #[snafu(display("Failed to convert data from {from} to {to}"))]
    TypeConversionFailed {
        from: String,
        to: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Timestamp conversion failed: {message}"))]
    TimestampConversionFailed {
        message: String,
        timestamp: Option<i64>,
        backtrace: Backtrace,
    },

    #[snafu(display("Data validation failed: {field} value is {value}"))]
    DataValidationFailed {
        field: String,
        value: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Stream data processing failed"))]
    StreamProcessingFailed { backtrace: Backtrace },

    #[snafu(display("Stream data format error"))]
    InvalidStreamDataFormat {
        expected_format: String,
        actual_data: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse kline '{symbol}' at interval '{interval}'"))]
    KlineDataParseFailed {
        symbol: String,
        interval: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse order '{order_id}'"))]
    OrderDataParseFailed {
        order_id: i64,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse position"))]
    PositionDataParseFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("Failed to parse deal '{deal_id}'"))]
    DealDataParseFailed {
        deal_id: i64,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse account info '{account_id}'"))]
    AccountInfoParseFailed {
        account_id: i32,
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for DataProcessorError {
    fn get_prefix(&self) -> &'static str {
        "DATA_PROCESSOR"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            DataProcessorError::JsonParseFailed { .. } => 1002,
            DataProcessorError::ArrayParseFailed { .. } => 1003,
            DataProcessorError::EnumParseFailed { .. } => 1004,

            // Data Structure Errors
            DataProcessorError::MissingField { .. } => 1005,
            DataProcessorError::ValueIsNone { .. } => 1006,
            DataProcessorError::InvalidFieldType { .. } => 1007,
            DataProcessorError::InvalidKlineArrayFormat { .. } => 1008,
            DataProcessorError::TypeConversionFailed { .. } => 1009,
            DataProcessorError::TimestampConversionFailed { .. } => 1010,

            // Data Validation Error
            DataProcessorError::DataValidationFailed { .. } => 1011,

            // Stream Data Processing Errors
            DataProcessorError::StreamProcessingFailed { .. } => 1013,
            DataProcessorError::InvalidStreamDataFormat { .. } => 1014,

            // Business Data Parsing Errors
            DataProcessorError::KlineDataParseFailed { .. } => 1015,
            DataProcessorError::OrderDataParseFailed { .. } => 1016,
            DataProcessorError::PositionDataParseFailed { .. } => 1017,
            DataProcessorError::DealDataParseFailed { .. } => 1018,
            DataProcessorError::AccountInfoParseFailed { .. } => 1019,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // Basic Parsing Errors
                DataProcessorError::JsonParseFailed { .. } => "JSON解析失败".to_string(),
                DataProcessorError::ArrayParseFailed { actual_type, .. } => {
                    format!("数组数据解析失败: 期望数组格式，实际类型 {}", actual_type)
                }
                DataProcessorError::EnumParseFailed { field, variant, .. } => {
                    format!("枚举解析失败，字段 '{}': 未知变体 '{}'", field, variant)
                }

                // Data Structure Errors
                DataProcessorError::MissingField { field, .. } => {
                    format!("JSON数据中缺少必需字段 '{}'", field)
                }
                DataProcessorError::ValueIsNone { field, .. } => {
                    format!("字段 '{}' 的值为空", field)
                }
                DataProcessorError::InvalidFieldType {
                    field, expected, actual, ..
                } => {
                    format!("字段 '{}' 数据类型无效: 期望 {}，实际 {}", field, expected, actual)
                }
                DataProcessorError::InvalidKlineArrayFormat { length, data, .. } => {
                    format!(
                        "K线数组格式无效: 期望6个元素 [时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量]，实际长度 {}，数据: {}",
                        length, data
                    )
                }

                // Data Conversion Errors
                DataProcessorError::TypeConversionFailed { from, to, .. } => {
                    format!("数据转换失败，从 {} 到 {}", from, to)
                }
                DataProcessorError::TimestampConversionFailed { message, timestamp, .. } => {
                    if let Some(ts) = timestamp {
                        format!("时间戳转换失败: {}, 时间戳: {}", message, ts)
                    } else {
                        format!("时间戳转换失败: {}", message)
                    }
                }

                // Data Validation Error
                DataProcessorError::DataValidationFailed { field, value, .. } => {
                    format!("数据验证失败: 字段 '{}' 值为 '{}'", field, value)
                }

                // Stream Data Processing Errors
                DataProcessorError::StreamProcessingFailed { .. } => {
                    format!("流数据处理失败")
                }
                DataProcessorError::InvalidStreamDataFormat {
                    expected_format,
                    actual_data,
                    ..
                } => {
                    format!("流数据格式错误: 期望格式 '{}'，实际数据 '{}'", expected_format, actual_data)
                }

                // Business Data Parsing Errors
                DataProcessorError::KlineDataParseFailed { symbol, interval, .. } => {
                    format!("解析K线数据失败: '{symbol}'  at interval '{interval}'")
                }
                DataProcessorError::OrderDataParseFailed { order_id, .. } => {
                    format!("解析订单数据失败: '{order_id}'")
                }
                DataProcessorError::PositionDataParseFailed { .. } => format!("解析持仓数据失败"),
                DataProcessorError::DealDataParseFailed { deal_id, .. } => {
                    format!("解析交易数据失败: '{deal_id}'")
                }
                DataProcessorError::AccountInfoParseFailed { account_id, .. } => {
                    format!("解析账户信息失败: '{account_id}'")
                }
            },
        }
    }
}
