use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::error_trait::StarRiverErrorTrait;
use crate::error::indicator_error::IndicatorError;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorNodeError {
    #[snafu(display("indicator node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("indicator node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("{config_name} should be greater than or equal to zero, but got {config_value}"))]
    ValueNotGreaterThanOrEqualToZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    #[snafu(display("{config_name} should be greater than zero, but got {config_value}"))]
    ValueNotGreaterThanZero {
        config_name: String,
        config_value: f64,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    IndicatorError { source: IndicatorError, backtrace: Backtrace },

    #[snafu(display("data source [{data_source}] parse failed. reason: [{source}]"))]
    DataSourceParseFailed {
        data_source: String,
        source: strum::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("get kline data failed"))]
    GetKlineDataFailed {
        source: Arc<dyn StarRiverErrorTrait + Send + Sync>,
        backtrace: Backtrace,
    },

    #[snafu(display("calculate indicator failed"))]
    CalculateIndicatorFailed {
        source: Arc<dyn StarRiverErrorTrait + Send + Sync>,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorNodeError
impl crate::error::error_trait::StarRiverErrorTrait for IndicatorNodeError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            IndicatorNodeError::ConfigFieldValueNull { .. } => 1001, // 指标节点回测配置字段值为空
            IndicatorNodeError::ConfigDeserializationFailed { .. } => 1002, // 指标节点回测配置反序列化失败
            IndicatorNodeError::ValueNotGreaterThanOrEqualToZero { .. } => 1003, // 指标节点回测配置值不能小于零
            IndicatorNodeError::ValueNotGreaterThanZero { .. } => 1004, // 指标节点回测配置值不能大于零
            IndicatorNodeError::IndicatorError { .. } => 1005,       // 指标错误
            IndicatorNodeError::DataSourceParseFailed { .. } => 1006, // 数据源解析失败
            IndicatorNodeError::GetKlineDataFailed { .. } => 1007,   // 获取K线数据失败
            IndicatorNodeError::CalculateIndicatorFailed { .. } => 1008, // 计算指标失败
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
            IndicatorNodeError::ConfigFieldValueNull { .. }
                | IndicatorNodeError::ConfigDeserializationFailed { .. }
                | IndicatorNodeError::ValueNotGreaterThanOrEqualToZero { .. }
                | IndicatorNodeError::ValueNotGreaterThanZero { .. }
                | IndicatorNodeError::IndicatorError { .. }
                | IndicatorNodeError::DataSourceParseFailed { .. }
                | IndicatorNodeError::GetKlineDataFailed { .. }
                | IndicatorNodeError::CalculateIndicatorFailed { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            IndicatorNodeError::IndicatorError { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }

            // For errors with external sources or no source
            IndicatorNodeError::ConfigFieldValueNull { .. }
            | IndicatorNodeError::ValueNotGreaterThanOrEqualToZero { .. }
            | IndicatorNodeError::ValueNotGreaterThanZero { .. } => vec![self.error_code()],

            // For errors with external sources that don't implement our trait
            IndicatorNodeError::ConfigDeserializationFailed { .. } | IndicatorNodeError::DataSourceParseFailed { .. } => {
                vec![self.error_code()]
            }
            IndicatorNodeError::GetKlineDataFailed { source, .. } | IndicatorNodeError::CalculateIndicatorFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                IndicatorNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("指标节点回测配置字段值为空: {}", field_name)
                }
                IndicatorNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("指标节点回测配置反序列化失败，原因: {}", source)
                }
                IndicatorNodeError::ValueNotGreaterThanOrEqualToZero {
                    config_name, config_value, ..
                } => {
                    format!("配置 {} 应该大于等于零，但得到了 {}", config_name, config_value)
                }
                IndicatorNodeError::ValueNotGreaterThanZero {
                    config_name, config_value, ..
                } => {
                    format!("配置 {} 应该大于零，但得到了 {}", config_name, config_value)
                }
                IndicatorNodeError::IndicatorError { source, .. } => {
                    format!("指标错误: {}", source)
                }
                IndicatorNodeError::DataSourceParseFailed { data_source, source, .. } => {
                    format!("数据源 [{}] 解析失败，原因: [{}]", data_source, source)
                }
                IndicatorNodeError::GetKlineDataFailed { source, .. } => {
                    format!("获取K线数据失败: {}", source)
                }
                IndicatorNodeError::CalculateIndicatorFailed { source, .. } => {
                    format!("计算指标失败: {}", source)
                }
            },
        }
    }
}
