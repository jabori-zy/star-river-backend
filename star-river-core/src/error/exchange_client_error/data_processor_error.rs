use crate::error::ErrorCode;
use crate::error::datetime_error::DateTimeError;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DataProcessorError {
    #[snafu(display("JSON parsing failed"))]
    JsonParsing {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Stream data processing failed: {message}"))]
    StreamProcessing {
        message: String,
        data_type: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to convert data from {from} to {to}"))]
    TypeConversion {
        from: String,
        to: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Required field '{field}' missing in JSON data"))]
    MissingField {
        field: String,
        context: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Value is None for field '{field}' in context '{context}'"))]
    ValueIsNone {
        field: String,
        context: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid data type for field '{field}': expected {expected}, got {actual}"))]
    InvalidFieldType {
        field: String,
        expected: String,
        actual: String,
        context: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse kline data: {message}"))]
    KlineDataParsing {
        message: String,
        symbol: Option<String>,
        interval: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse order data: {message}"))]
    OrderDataParsing {
        message: String,
        order_id: Option<i64>,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse position data: {message}"))]
    PositionDataParsing {
        message: String,
        position_id: Option<i64>,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse deal data: {message}"))]
    DealDataParsing {
        message: String,
        deal_id: Option<i64>,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse account info: {message}"))]
    AccountInfoParsing {
        message: String,
        account_id: Option<i32>,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Array data parsing failed: expected array format, got {actual_type}"))]
    ArrayParsing {
        actual_type: String,
        context: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Invalid kline array format: expected 6 elements [timestamp, open, high, low, close, volume], got {length}"
    ))]
    InvalidKlineArrayFormat {
        length: usize,
        data: String,
        backtrace: Backtrace,
    },

    #[snafu(display("{message}. {field} value is {value}"))]
    DataValidation {
        message: String,
        context: Option<String>,
        field: String,
        value: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Enum parsing failed for field '{field}': unknown variant '{variant}'"))]
    EnumParsing {
        field: String,
        variant: String,
        valid_variants: Vec<String>,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Stream data format error: {message}"))]
    StreamDataFormat {
        message: String,
        expected_format: Option<String>,
        actual_data: Option<String>,
        backtrace: Backtrace,
    },

    #[snafu(display("Timestamp conversion failed: {message}"))]
    TimestampConversion {
        message: String,
        timestamp: Option<i64>,
        backtrace: Backtrace,
    },

    #[snafu(display("Data processing internal error: {message}"))]
    Internal { message: String, backtrace: Backtrace },

    #[snafu(transparent)]
    DateTime {
        source: DateTimeError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for DataProcessorError
impl crate::error::error_trait::StarRiverErrorTrait for DataProcessorError {
    fn get_prefix(&self) -> &'static str {
        "DATA_PROCESSOR"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // JSON and serialization errors (1001-1002)
            DataProcessorError::JsonParsing { .. } => 1001,
            DataProcessorError::StreamProcessing { .. } => 1002,
            // Field and data structure errors (1003-1007)
            DataProcessorError::MissingField { .. } => 1003,
            DataProcessorError::ValueIsNone { .. } => 1004,
            DataProcessorError::InvalidFieldType { .. } => 1005,
            DataProcessorError::ArrayParsing { .. } => 1006,
            DataProcessorError::InvalidKlineArrayFormat { .. } => 1007,

            // Type conversion and validation errors (1007-1010)
            DataProcessorError::TypeConversion { .. } => 1007,
            DataProcessorError::DataValidation { .. } => 1008,
            DataProcessorError::EnumParsing { .. } => 1009,
            DataProcessorError::TimestampConversion { .. } => 1010,

            // Specific data parsing errors (1011-1016)
            DataProcessorError::KlineDataParsing { .. } => 1011,
            DataProcessorError::OrderDataParsing { .. } => 1012,
            DataProcessorError::PositionDataParsing { .. } => 1013,
            DataProcessorError::DealDataParsing { .. } => 1014,
            DataProcessorError::AccountInfoParsing { .. } => 1015,
            DataProcessorError::StreamDataFormat { .. } => 1016,

            // Internal errors (1017)
            DataProcessorError::Internal { .. } => 1017,
            DataProcessorError::DateTime { .. } => 1018,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let mut ctx = HashMap::new();
        match self {
            DataProcessorError::StreamProcessing {
                data_type: Some(dt), ..
            } => {
                ctx.insert("data_type", dt.clone());
            }
            DataProcessorError::MissingField { field, context, .. } => {
                ctx.insert("field", field.clone());
                if let Some(c) = context {
                    ctx.insert("context", c.clone());
                }
            }
            DataProcessorError::InvalidFieldType {
                field,
                expected,
                actual,
                context,
                ..
            } => {
                ctx.insert("field", field.clone());
                ctx.insert("expected", expected.clone());
                ctx.insert("actual", actual.clone());
                if let Some(c) = context {
                    ctx.insert("context", c.clone());
                }
            }
            DataProcessorError::KlineDataParsing { symbol, interval, .. } => {
                if let Some(sym) = symbol {
                    ctx.insert("symbol", sym.clone());
                }
                if let Some(int) = interval {
                    ctx.insert("interval", int.clone());
                }
            }
            DataProcessorError::OrderDataParsing { order_id, .. } => {
                if let Some(id) = order_id {
                    ctx.insert("order_id", id.to_string());
                }
            }
            DataProcessorError::PositionDataParsing { position_id, .. } => {
                if let Some(id) = position_id {
                    ctx.insert("position_id", id.to_string());
                }
            }
            DataProcessorError::DealDataParsing { deal_id, .. } => {
                if let Some(id) = deal_id {
                    ctx.insert("deal_id", id.to_string());
                }
            }
            DataProcessorError::AccountInfoParsing { account_id, .. } => {
                if let Some(id) = account_id {
                    ctx.insert("account_id", id.to_string());
                }
            }
            DataProcessorError::EnumParsing { field, variant, .. } => {
                ctx.insert("field", field.clone());
                ctx.insert("variant", variant.clone());
            }
            DataProcessorError::TimestampConversion { timestamp, .. } => {
                if let Some(ts) = timestamp {
                    ctx.insert("timestamp", ts.to_string());
                }
            }
            DataProcessorError::DateTime { source, .. } => {
                ctx.insert("source", source.to_string());
            }
            _ => {}
        }
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            // Most data processing errors are not recoverable as they indicate
            // structural problems with the data format or logic errors
            DataProcessorError::StreamProcessing { .. }
                | DataProcessorError::KlineDataParsing { .. }
                | DataProcessorError::OrderDataParsing { .. }
                | DataProcessorError::PositionDataParsing { .. }
                | DataProcessorError::DealDataParsing { .. }
                | DataProcessorError::AccountInfoParsing { .. }
                | DataProcessorError::StreamDataFormat { .. }
                | DataProcessorError::TimestampConversion { .. }
                | DataProcessorError::DateTime { .. } // JSON parsing, missing fields, type conversions, etc. are typically not recoverable
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // Most DataProcessorError variants have external source types
        // (serde_json::Error, strum::ParseError) that don't implement our trait
        vec![self.error_code()]
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                DataProcessorError::JsonParsing { .. } => "JSON解析失败".to_string(),
                DataProcessorError::StreamProcessing { message, data_type, .. } => {
                    if let Some(dt) = data_type {
                        format!("流数据处理失败: {}, 数据类型: {}", message, dt)
                    } else {
                        format!("流数据处理失败: {}", message)
                    }
                }
                DataProcessorError::TypeConversion { from, to, .. } => {
                    format!("数据转换失败，从 {} 到 {}", from, to)
                }
                DataProcessorError::MissingField { field, context, .. } => {
                    if let Some(ctx) = context {
                        format!("JSON数据中缺少必需字段 '{}' (上下文: {})", field, ctx)
                    } else {
                        format!("JSON数据中缺少必需字段 '{}'", field)
                    }
                }
                DataProcessorError::ValueIsNone { field, context, .. } => {
                    format!("字段 '{}' 在上下文 '{}' 中的值为空", field, context)
                }
                DataProcessorError::InvalidFieldType {
                    field,
                    expected,
                    actual,
                    context,
                    ..
                } => {
                    if let Some(ctx) = context {
                        format!(
                            "字段 '{}' 数据类型无效: 期望 {}，实际 {} (上下文: {})",
                            field, expected, actual, ctx
                        )
                    } else {
                        format!("字段 '{}' 数据类型无效: 期望 {}，实际 {}", field, expected, actual)
                    }
                }
                DataProcessorError::KlineDataParsing {
                    message,
                    symbol,
                    interval,
                    ..
                } => {
                    let mut msg = format!("解析K线数据失败: {}", message);
                    if let Some(sym) = symbol {
                        msg.push_str(&format!(", 交易品种: {}", sym));
                    }
                    if let Some(int) = interval {
                        msg.push_str(&format!(", 时间间隔: {}", int));
                    }
                    msg
                }
                DataProcessorError::OrderDataParsing { message, order_id, .. } => {
                    if let Some(id) = order_id {
                        format!("解析订单数据失败: {}, 订单ID: {}", message, id)
                    } else {
                        format!("解析订单数据失败: {}", message)
                    }
                }
                DataProcessorError::PositionDataParsing {
                    message, position_id, ..
                } => {
                    if let Some(id) = position_id {
                        format!("解析持仓数据失败: {}, 持仓ID: {}", message, id)
                    } else {
                        format!("解析持仓数据失败: {}", message)
                    }
                }
                DataProcessorError::DealDataParsing { message, deal_id, .. } => {
                    if let Some(id) = deal_id {
                        format!("解析交易数据失败: {}, 交易ID: {}", message, id)
                    } else {
                        format!("解析交易数据失败: {}", message)
                    }
                }
                DataProcessorError::AccountInfoParsing {
                    message, account_id, ..
                } => {
                    if let Some(id) = account_id {
                        format!("解析账户信息失败: {}, 账户ID: {}", message, id)
                    } else {
                        format!("解析账户信息失败: {}", message)
                    }
                }
                DataProcessorError::ArrayParsing {
                    actual_type, context, ..
                } => {
                    format!(
                        "数组数据解析失败: 期望数组格式，实际类型 {} (上下文: {})",
                        actual_type, context
                    )
                }
                DataProcessorError::InvalidKlineArrayFormat { length, data, .. } => {
                    format!(
                        "K线数组格式无效: 期望6个元素 [时间戳, 开盘价, 最高价, 最低价, 收盘价, 成交量]，实际长度 {}，数据: {}",
                        length, data
                    )
                }
                DataProcessorError::DataValidation {
                    message,
                    context,
                    field,
                    value,
                    ..
                } => {
                    if let Some(ctx) = context {
                        format!("{}. 字段 {} 值为 {} (上下文: {})", message, field, value, ctx)
                    } else {
                        format!("{}. 字段 {} 值为 {}", message, field, value)
                    }
                }
                DataProcessorError::EnumParsing {
                    field,
                    variant,
                    valid_variants,
                    ..
                } => {
                    format!(
                        "枚举解析失败，字段 '{}': 未知变体 '{}', 有效变体: [{}]",
                        field,
                        variant,
                        valid_variants.join(", ")
                    )
                }
                DataProcessorError::StreamDataFormat {
                    message,
                    expected_format,
                    actual_data,
                    ..
                } => {
                    let mut msg = format!("流数据格式错误: {}", message);
                    if let Some(expected) = expected_format {
                        msg.push_str(&format!(", 期望格式: {}", expected));
                    }
                    if let Some(actual) = actual_data {
                        msg.push_str(&format!(", 实际数据: {}", actual));
                    }
                    msg
                }
                DataProcessorError::TimestampConversion { message, timestamp, .. } => {
                    if let Some(ts) = timestamp {
                        format!("时间戳转换失败: {}, 时间戳: {}", message, ts)
                    } else {
                        format!("时间戳转换失败: {}", message)
                    }
                }
                DataProcessorError::Internal { message, .. } => {
                    format!("数据处理内部错误: {}", message)
                }
                DataProcessorError::DateTime { source, .. } => {
                    format!("时间戳转换失败: {}", source)
                }
            },
        }
    }
}
