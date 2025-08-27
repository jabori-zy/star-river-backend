pub mod mt5;
mod data_processor_error;



use thiserror::Error;
pub use mt5::Mt5Error;
pub use data_processor_error::DataProcessorError;
use super::ErrorCode;


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
    /// Returns the error prefix for exchange client errors
    pub fn get_prefix(&self) -> &'static str {
        "EXCHANGE_CLIENT"
    }
    
    /// Returns a string error code for exchange client errors (format: EXCHANGE_CLIENT_NNNN or nested error codes)
    pub fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeClientError::MetaTrader5(mt5_err) => mt5_err.error_code(),
            ExchangeClientError::DataProcessor(data_err) => data_err.error_code(),
            
            // For direct exchange client errors, use EXCHANGE_CLIENT prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // Exchange-specific errors (1001)
                    ExchangeClientError::Binance(_) => 1001,
                    
                    // Network and connection errors (1002-1005)
                    ExchangeClientError::Connection(_) => 1002,
                    ExchangeClientError::Network(_) => 1003,
                    ExchangeClientError::WebSocket(_) => 1004,
                    ExchangeClientError::Timeout(_) => 1005,
                    
                    // Authentication and authorization errors (1006-1007)
                    ExchangeClientError::Authentication(_) => 1006,
                    ExchangeClientError::RateLimit(_) => 1007,
                    
                    // Data and parameter errors (1008-1010)
                    ExchangeClientError::InvalidParameters(_) => 1008,
                    ExchangeClientError::MarketData(_) => 1009,
                    ExchangeClientError::Serialization(_) => 1010,
                    
                    // Trading operations errors (1011-1013)
                    ExchangeClientError::OrderManagement(_) => 1011,
                    ExchangeClientError::PositionManagement(_) => 1012,
                    ExchangeClientError::AccountManagement(_) => 1013,
                    
                    // System errors (1014-1016)
                    ExchangeClientError::Configuration(_) => 1014,
                    ExchangeClientError::UnsupportedExchange(_) => 1015,
                    ExchangeClientError::NotImplemented(_) => 1016,
                    ExchangeClientError::Internal(_) => 1017,
                    
                    // This should never happen due to outer match, but needed for completeness
                    ExchangeClientError::MetaTrader5(_) | ExchangeClientError::DataProcessor(_) => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

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

// Implement the StarRiverErrorTrait for ExchangeClientError
impl super::error_trait::StarRiverErrorTrait for ExchangeClientError {
    fn get_prefix(&self) -> &'static str {
        // For nested errors, delegate to the inner error's prefix
        match self {
            ExchangeClientError::MetaTrader5(mt5_err) => mt5_err.get_prefix(),
            ExchangeClientError::DataProcessor(data_err) => data_err.get_prefix(),
            _ => self.get_prefix(),
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn category(&self) -> &'static str {
        "exchange"
    }
    
    fn is_retriable(&self) -> bool {
        matches!(self,
            ExchangeClientError::Connection(_) |
            ExchangeClientError::Network(_) |
            ExchangeClientError::WebSocket(_) |
            ExchangeClientError::Timeout(_) |
            ExchangeClientError::RateLimit(_)
        )
    }
    
    fn is_client_error(&self) -> bool {
        matches!(self,
            ExchangeClientError::Authentication(_) |
            ExchangeClientError::InvalidParameters(_) |
            ExchangeClientError::UnsupportedExchange(_)
        )
    }
    
    fn message(&self) -> &str {
        match self {
            ExchangeClientError::Binance(msg) |
            ExchangeClientError::Connection(msg) |
            ExchangeClientError::Authentication(msg) |
            ExchangeClientError::RateLimit(msg) |
            ExchangeClientError::InvalidParameters(msg) |
            ExchangeClientError::MarketData(msg) |
            ExchangeClientError::OrderManagement(msg) |
            ExchangeClientError::PositionManagement(msg) |
            ExchangeClientError::AccountManagement(msg) |
            ExchangeClientError::WebSocket(msg) |
            ExchangeClientError::Serialization(msg) |
            ExchangeClientError::Network(msg) |
            ExchangeClientError::Timeout(msg) |
            ExchangeClientError::Configuration(msg) |
            ExchangeClientError::UnsupportedExchange(msg) |
            ExchangeClientError::NotImplemented(msg) |
            ExchangeClientError::Internal(msg) => msg,
            ExchangeClientError::MetaTrader5(_) => "MT5 error occurred",
            ExchangeClientError::DataProcessor(_) => "Data processing error occurred",
        }
    }
}

// Implement ErrorContext trait for ExchangeClientError
impl<T> super::error_trait::ErrorContext<T, ExchangeClientError> for Result<T, ExchangeClientError> {
    fn with_context<F>(self, f: F) -> Result<T, ExchangeClientError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            ExchangeClientError::Internal(format!("{}: {}", context, e))
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, ExchangeClientError> {
        self.map_err(|e| {
            ExchangeClientError::Internal(format!("Operation '{}': {}", operation, e))
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, ExchangeClientError> {
        self.map_err(|e| {
            ExchangeClientError::Internal(format!("{} '{}': {}", resource_type, resource_id, e))
        })
    }
}