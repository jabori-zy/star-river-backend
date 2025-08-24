use thiserror::Error;
use crate::metatrader5::mt5_http_client_error::Mt5HttpClientError;
use crate::data_processor_error::DataProcessorError;

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