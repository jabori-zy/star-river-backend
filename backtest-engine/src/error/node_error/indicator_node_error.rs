use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode, generate_error_code_chain};
use ta_lib::error::TaLibError;
use snafu::{Backtrace, Snafu};
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorNodeError {

    #[snafu(transparent)]
    IndicatorError { source: TaLibError, backtrace: Backtrace },

    

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
impl StarRiverErrorTrait for IndicatorNodeError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IndicatorNodeError::IndicatorError { .. } => 1001,       // indicator error
            IndicatorNodeError::DataSourceParseFailed { .. } => 1002, // data source parse failed
            IndicatorNodeError::GetKlineDataFailed { .. } => 1003,   // get kline data failed
            IndicatorNodeError::CalculateIndicatorFailed { .. } => 1004, // calculate indicator failed
        };

        format!("{}_{:04}", prefix, code)
    }


    fn http_status_code(&self) -> StatusCode {
        match self {
            // transparent errors - return source http status code
            IndicatorNodeError::IndicatorError { source, .. } => StatusCode::INTERNAL_SERVER_ERROR,
            IndicatorNodeError::GetKlineDataFailed { source, .. } => source.http_status_code(),
            IndicatorNodeError::CalculateIndicatorFailed { source, .. } => source.http_status_code(),
            
            // non-transparent errors - use custom http status code
            IndicatorNodeError::DataSourceParseFailed { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            IndicatorNodeError::IndicatorError { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(source.error_code());
                chain
            },
            IndicatorNodeError::GetKlineDataFailed { source, .. } |
            IndicatorNodeError::CalculateIndicatorFailed { source, .. } => generate_error_code_chain(source.as_ref()),

            // For errors with external sources that don't implement our trait
            _ => vec![self.error_code()]
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // transparent errors - return source message directly
                IndicatorNodeError::IndicatorError { source, .. } => source.error_message(&language.to_string()),
                IndicatorNodeError::GetKlineDataFailed { source, .. } => source.error_message(language),
                
                IndicatorNodeError::CalculateIndicatorFailed { source, .. } => source.error_message(language),
                
                // non-transparent errors - use custom message
                IndicatorNodeError::DataSourceParseFailed { data_source, source, .. } => {
                    format!("数据源 [{}] 解析失败，原因: [{}]", data_source, source)
                }

                
            },
        }
    }
}
