use std::sync::Arc;

use event_center::EventCenterError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};
use strategy_core::error::{NodeError, NodeStateMachineError};
use ta_lib::{IndicatorConfig, error::TaLibError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    TaLibError { source: TaLibError, backtrace: Backtrace },

    #[snafu(transparent)]
    EventCenterError { source: EventCenterError, backtrace: Backtrace },

    #[snafu(transparent)]
    NodeStateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] indicator engine error: {source}"))]
    IndicatorEngineError {
        node_name: NodeName,
        source: Arc<dyn StarRiverErrorTrait + Send + Sync>,
        backtrace: Backtrace,
    },

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

    #[snafu(display("@[{node_name}] calculate indicator failed: {source}"))]
    CalculateIndicatorFailed {
        node_name: NodeName,
        source: Arc<dyn StarRiverErrorTrait + Send + Sync>,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] calculate indicator result is empty. indicator_config: {:#?}", indicator_config))]
    CalculateResultEmpty {
        node_name: NodeName,
        indicator_config: IndicatorConfig,
    },

    #[snafu(display("@[{node_name}] symbol is not configured"))]
    ExchangeModeNotConfigured { node_name: NodeName, backtrace: Backtrace },

    #[snafu(display("@[{node_name}] file mode is not configured"))]
    FileModeNotConfigured { node_name: NodeName, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for IndicatorNodeError
impl StarRiverErrorTrait for IndicatorNodeError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IndicatorNodeError::NodeError { .. } => 1001,                 // node error
            IndicatorNodeError::TaLibError { .. } => 1002,                // indicator error
            IndicatorNodeError::EventCenterError { .. } => 1003,          // event center error
            IndicatorNodeError::NodeStateMachineError { .. } => 1004,     // node state machine error
            IndicatorNodeError::IndicatorEngineError { .. } => 1005,      // indicator engine error
            IndicatorNodeError::DataSourceParseFailed { .. } => 1006,     // data source parse failed
            IndicatorNodeError::GetKlineDataFailed { .. } => 1007,        // get kline data failed
            IndicatorNodeError::CalculateIndicatorFailed { .. } => 1008,  // calculate indicator failed
            IndicatorNodeError::CalculateResultEmpty { .. } => 1009,      // calculate result empty
            IndicatorNodeError::ExchangeModeNotConfigured { .. } => 1010, // exchange mode is not configured
            IndicatorNodeError::FileModeNotConfigured { .. } => 1011,     // file mode is not configured
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            // transparent errors - return source http status code
            IndicatorNodeError::NodeError { source, .. } => source.http_status_code(),
            IndicatorNodeError::TaLibError { source, .. } => source.http_status_code(),
            IndicatorNodeError::EventCenterError { source, .. } => source.http_status_code(),
            IndicatorNodeError::NodeStateMachineError { source, .. } => source.http_status_code(),
            IndicatorNodeError::IndicatorEngineError { source, .. } => source.http_status_code(),
            IndicatorNodeError::GetKlineDataFailed { source, .. } => source.http_status_code(),
            IndicatorNodeError::CalculateIndicatorFailed { source, .. } => source.http_status_code(),
            IndicatorNodeError::CalculateResultEmpty { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            IndicatorNodeError::ExchangeModeNotConfigured { .. } => StatusCode::BAD_REQUEST,
            IndicatorNodeError::FileModeNotConfigured { .. } => StatusCode::BAD_REQUEST,
            // non-transparent errors - use custom http status code
            IndicatorNodeError::DataSourceParseFailed { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            IndicatorNodeError::NodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            IndicatorNodeError::TaLibError { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(source.error_code());
                chain
            }
            IndicatorNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            IndicatorNodeError::IndicatorEngineError { source, .. }
            | IndicatorNodeError::GetKlineDataFailed { source, .. }
            | IndicatorNodeError::CalculateIndicatorFailed { source, .. } => generate_error_code_chain(source.as_ref(), self.error_code()),

            // For errors with external sources that don't implement our trait
            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                IndicatorNodeError::NodeError { source, .. } => source.error_message(language),
                // transparent errors - return source message directly
                IndicatorNodeError::TaLibError { source, .. } => source.error_message(&language.to_string()),
                IndicatorNodeError::EventCenterError { source, .. } => source.error_message(language),
                IndicatorNodeError::NodeStateMachineError { source, .. } => source.error_message(language),
                IndicatorNodeError::GetKlineDataFailed { source, .. } => source.error_message(language),
                IndicatorNodeError::IndicatorEngineError { node_name, source, .. } => {
                    format!("@[{node_name}] 指标引擎错误: {}", source.error_message(language))
                }
                IndicatorNodeError::CalculateIndicatorFailed { node_name, source, .. } => {
                    format!("@[{node_name}] 计算指标失败: {}", source.error_message(language))
                }

                // non-transparent errors - use custom message
                IndicatorNodeError::DataSourceParseFailed { data_source, source, .. } => {
                    format!("数据源 [{}] 解析失败，原因: [{}]", data_source, source)
                }
                IndicatorNodeError::CalculateResultEmpty {
                    node_name,
                    indicator_config,
                    ..
                } => {
                    format!("@[{node_name}] 计算指标结果为空. 指标配置: {:#?}", indicator_config)
                }
                IndicatorNodeError::ExchangeModeNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 交易所模式未配置")
                }
                IndicatorNodeError::FileModeNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 文件模式未配置")
                }
            },
        }
    }
}
