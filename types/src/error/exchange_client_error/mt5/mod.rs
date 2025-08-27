pub mod mt5_http_client_error;

pub use mt5_http_client_error::*;


use thiserror::Error;
use super::data_processor_error::DataProcessorError;
pub use mt5_http_client_error::Mt5HttpClientError;
use crate::error::ErrorCode;

#[derive(Error, Debug)]
pub enum Mt5Error {
    #[error("MT5 HTTP client error: {0}")]
    HttpClient(#[from] Mt5HttpClientError),
    
    #[error("MT5 WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("MT5 data processor error: {0}")]
    DataProcessor(#[from] DataProcessorError),
    
    #[error("MT5 connection error: {0}")]
    Connection(String),
    
    #[error("MT5 initialization error: {0}")]
    Initialization(String),
    
    #[error("MT5 configuration error: {0}")]
    Configuration(String),
    
    #[error("MT5 server error: {0}")]
    Server(String),
    
    #[error("MT5 timeout error: {0}")]
    Timeout(String),
    
    #[error("MT5 authentication error: {0}")]
    Authentication(String),
    
    #[error("MT5 validation error: {0}")]
    Validation(String),
    
    #[error("MT5 internal error: {0}")]
    Internal(String),
}

impl Mt5Error {
    /// Returns the error prefix for MT5 errors
    pub fn get_prefix(&self) -> &'static str {
        "MT5"
    }
    
    /// Returns a string error code for MT5 errors (format: MT5_NNNN)
    pub fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            Mt5Error::HttpClient(http_err) => http_err.error_code(),
            Mt5Error::DataProcessor(data_err) => data_err.error_code(),
            
            // For direct MT5 errors, use MT5 prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // WebSocket errors (1001)
                    Mt5Error::WebSocket(_) => 1001,
                    
                    // Connection and initialization errors (1002-1004)
                    Mt5Error::Connection(_) => 1002,
                    Mt5Error::Initialization(_) => 1003,
                    Mt5Error::Configuration(_) => 1004,
                    
                    // Server and service errors (1005-1008)
                    Mt5Error::Server(_) => 1005,
                    Mt5Error::Timeout(_) => 1006,
                    Mt5Error::Authentication(_) => 1007,
                    Mt5Error::Validation(_) => 1008,
                    
                    // Internal errors (1009)
                    Mt5Error::Internal(_) => 1009,
                    
                    // This should never happen due to outer match, but needed for completeness
                    Mt5Error::HttpClient(_) | Mt5Error::DataProcessor(_) => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    pub fn websocket<S: Into<String>>(message: S) -> Self {
        Self::WebSocket(message.into())
    }
    
    pub fn data_processor(error: DataProcessorError) -> Self {
        Self::DataProcessor(error)
    }
    
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection(message.into())
    }
    
    pub fn initialization<S: Into<String>>(message: S) -> Self {
        Self::Initialization(message.into())
    }
    
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration(message.into())
    }
    
    pub fn server<S: Into<String>>(message: S) -> Self {
        Self::Server(message.into())
    }
    
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }
    
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication(message.into())
    }
    
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }
    
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for Mt5Error {
    fn get_prefix(&self) -> &'static str {
        // For nested errors, delegate to the inner error's prefix
        match self {
            Mt5Error::HttpClient(http_err) => http_err.get_prefix(),
            Mt5Error::DataProcessor(data_err) => data_err.get_prefix(),
            _ => self.get_prefix(),
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn category(&self) -> &'static str {
        "mt5"
    }
    
    fn is_retriable(&self) -> bool {
        matches!(self,
            Mt5Error::Connection(_) |
            Mt5Error::WebSocket(_) |
            Mt5Error::Server(_) |
            Mt5Error::Timeout(_)
        )
    }
    
    fn is_client_error(&self) -> bool {
        matches!(self,
            Mt5Error::Authentication(_) |
            Mt5Error::Validation(_) |
            Mt5Error::Configuration(_)
        )
    }
    
    fn message(&self) -> &str {
        match self {
            Mt5Error::WebSocket(msg) |
            Mt5Error::Connection(msg) |
            Mt5Error::Initialization(msg) |
            Mt5Error::Configuration(msg) |
            Mt5Error::Server(msg) |
            Mt5Error::Timeout(msg) |
            Mt5Error::Authentication(msg) |
            Mt5Error::Validation(msg) |
            Mt5Error::Internal(msg) => msg,
            Mt5Error::HttpClient(_) => "MT5 HTTP client error occurred",
            Mt5Error::DataProcessor(_) => "MT5 data processing error occurred",
        }
    }
}

// Implement ErrorContext trait for Mt5Error
impl<T> crate::error::error_trait::ErrorContext<T, Mt5Error> for Result<T, Mt5Error> {
    fn with_context<F>(self, f: F) -> Result<T, Mt5Error>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            Mt5Error::Internal(format!("{}: {}", context, e))
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, Mt5Error> {
        self.map_err(|e| {
            Mt5Error::Internal(format!("MT5 Operation '{}': {}", operation, e))
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, Mt5Error> {
        self.map_err(|e| {
            Mt5Error::Internal(format!("MT5 {} '{}': {}", resource_type, resource_id, e))
        })
    }
}