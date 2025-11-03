// third-party
use snafu::{Backtrace, Snafu};

// workspace crate
use star_river_core::error::{ErrorCode, ErrorLanguage, ReqwestError, StarRiverErrorTrait};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeError {
    #[snafu(display("http client not created"))]
    HttpClientNotCreated { backtrace: Backtrace },

    #[snafu(display("http request failed: url: {url}, source: {source}"))]
    HttpRequestFailed {
        url: String,
        source: ReqwestError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl StarRiverErrorTrait for ExchangeError {
    fn get_prefix(&self) -> &'static str {
        "EXCHANGE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            ExchangeError::HttpClientNotCreated { .. } => 1001,  // 客户端未创建
            ExchangeError::HttpRequestFailed { .. } => 1002,     // HTTP请求失败
        };
        format!("{}_{:04}", prefix, code)
    }


    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ExchangeError::HttpClientNotCreated { .. } => {
                    format!("客户端未创建")
                }
                ExchangeError::HttpRequestFailed { url, source, .. } => {
                    format!("HTTP请求失败: url: {}, source: {}", url, source)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // CreateIndicatorFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            ExchangeError::HttpClientNotCreated { .. } |
            ExchangeError::HttpRequestFailed { .. } => vec![self.error_code()],
        }
    }
}
