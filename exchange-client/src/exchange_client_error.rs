use thiserror::Error;
use crate::metatrader5::mt5_error::Mt5Error;
use crate::data_processor_error::DataProcessorError;

#[derive(Error, Debug)]
pub enum ExchangeClientError {
    #[error("MetaTrader5 error: {0}")]
    MetaTrader5(#[from] Mt5Error),
    
    #[error("Data processor error: {0}")]
    DataProcessor(#[from] DataProcessorError),
    
    #[error("Binance error: {0}")]
    Binance(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Market data error: {0}")]
    MarketData(String),
    
    #[error("Order management error: {0}")]
    OrderManagement(String),
    
    #[error("Position management error: {0}")]
    PositionManagement(String),
    
    #[error("Account management error: {0}")]
    AccountManagement(String),
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Exchange not supported: {0}")]
    UnsupportedExchange(String),
    
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ExchangeClientError {
    pub fn binance<S: Into<String>>(message: S) -> Self {
        Self::Binance(message.into())
    }
    
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection(message.into())
    }
    
    pub fn authentication<S: Into<String>>(message: S) -> Self {
        Self::Authentication(message.into())
    }
    
    pub fn rate_limit<S: Into<String>>(message: S) -> Self {
        Self::RateLimit(message.into())
    }
    
    pub fn invalid_parameters<S: Into<String>>(message: S) -> Self {
        Self::InvalidParameters(message.into())
    }
    
    pub fn market_data<S: Into<String>>(message: S) -> Self {
        Self::MarketData(message.into())
    }
    
    pub fn order_management<S: Into<String>>(message: S) -> Self {
        Self::OrderManagement(message.into())
    }
    
    pub fn position_management<S: Into<String>>(message: S) -> Self {
        Self::PositionManagement(message.into())
    }
    
    pub fn account_management<S: Into<String>>(message: S) -> Self {
        Self::AccountManagement(message.into())
    }
    
    pub fn websocket<S: Into<String>>(message: S) -> Self {
        Self::WebSocket(message.into())
    }
    
    pub fn serialization<S: Into<String>>(message: S) -> Self {
        Self::Serialization(message.into())
    }
    
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network(message.into())
    }
    
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }
    
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration(message.into())
    }
    
    pub fn unsupported_exchange<S: Into<String>>(message: S) -> Self {
        Self::UnsupportedExchange(message.into())
    }
    
    pub fn not_implemented<S: Into<String>>(message: S) -> Self {
        Self::NotImplemented(message.into())
    }
    
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }
}

// Conversion from common error types
impl From<String> for ExchangeClientError {
    fn from(err: String) -> Self {
        Self::Internal(err)
    }
}

impl From<&str> for ExchangeClientError {
    fn from(err: &str) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ExchangeClientError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<reqwest::Error> for ExchangeClientError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err.to_string())
        } else if err.is_connect() {
            Self::Connection(err.to_string())
        } else {
            Self::Network(err.to_string())
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ExchangeClientError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(err.to_string())
    }
}

// Helper trait for easy error context
pub trait ExchangeClientErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T, ExchangeClientError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ExchangeClientErrorContext<T> for Result<T, E>
where
    E: Into<ExchangeClientError>,
{
    fn with_context<F>(self, f: F) -> Result<T, ExchangeClientError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            ExchangeClientError::Internal(format!("{}: {}", context, base_error))
        })
    }
}