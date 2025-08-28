use thiserror::Error;
use super::data_processor_error::DataProcessorError;
use crate::error::ErrorCode;
use crate::custom_type::AccountId;

pub type MT5ErrorCode = i64;

#[derive(Error, Debug)]
pub enum Mt5Error {
    #[error("http error: message={message}, terminal_id={terminal_id}, port={port}")]
    Http {
        message: String,
        terminal_id: i32,
        port: u16,
        #[source]
        source: reqwest::Error,
    },

    #[error("no success field in the response")]
    NoSuccessFieldInResponse,

    #[error("http client not created: terminal_id={terminal_id}, port={port}")]
    HttpClientNotCreated {
        terminal_id: i32,
        port: u16,
    },
    
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    
    #[error("failed to initialize terminal: {0}")]
    InitializeTerminal(String),
    
    #[error("failed to get terminal info: {0}")]
    GetTerminalInfo(String),

    #[error("failed to get symbol list: {0}")]
    GetSymbolList(String),
    
    #[error("failed to get kline data for symbol '{symbol}': {message}")]
    GetKlineData { 
        symbol: String, 
        message: String, 
        code: Option<MT5ErrorCode>
    },
    
    #[error("failed to create order for symbol '{symbol}': {message}")]
    CreateOrder { 
        symbol: String, 
        message: String,
        code: Option<MT5ErrorCode>
    },
    
    #[error("failed to get order {order_id}: {message}")]
    GetOrder { 
        order_id: i64, 
        message: String 
    },
    
    #[error("failed to get position {position_id}: {message}")]
    GetPosition {
        position_id: i64, 
        message: String 
    },
    
    #[error("failed to get deal: {message}")]
    GetDeal {
        message: String,
        deal_id: Option<i64>,
        position_id: Option<i64>,
        order_id: Option<i64>,
    },
    
    #[error("failed to get position number for symbol '{symbol}': {message}")]
    GetPositionNumber {
        symbol: String, 
        message: String 
    },
    
    #[error("failed to get account info: message={message}, terminal_id={terminal_id}, port={port}")]
    GetAccountInfo {
        message: String,
        terminal_id: i32,
        port: u16,
    },
    
    #[error("ping failed: {message}, terminal_id={terminal_id}, port={port}")]
    Ping {
        message: String,
        terminal_id: i32,
        port: u16,
        #[source]
        source: Option<reqwest::Error>
    },
    
    #[error("metatrader5 websocket error: {message}, account_id: {account_id}, url: {url}")]
    WebSocket{
        message: String,
        account_id: AccountId,
        url: String,
        #[source]
        source: tokio_tungstenite::tungstenite::error::Error,
    },
    
    #[error(transparent)]
    DataProcessor(#[from] DataProcessorError),
    
    #[error("metatrader5 connection error: message={message}, terminal_id={terminal_id}, port={port}")]
    Connection{
        message: String,
        terminal_id: i32,
        port: u16,
    },
    
    #[error("metatrader5 initialization error: {0}")]
    Initialization(String),
    
    #[error("metatrader5 configuration error: {0}")]
    Configuration(String),
    
    #[error("metatrader5 server error: {0}")]
    Server(String),
    
    #[error("metatrader5 timeout error: {0}")]
    Timeout(String),
    
    #[error("metatrader5 authentication error: {0}")]
    Authentication(String),
    
    #[error("metatrader5 validation error: {0}")]
    Validation(String),
    
    #[error("metatrader5 internal error: {0}")]
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
            Mt5Error::DataProcessor(data_err) => data_err.error_code(),
            
            // For direct MT5 errors, use MT5 prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // HTTP and JSON errors (1001-1002)
                    Mt5Error::Http { .. } => 1001,
                    Mt5Error::NoSuccessFieldInResponse => 1002,
                    Mt5Error::HttpClientNotCreated { .. } => 1003,
                    Mt5Error::Json(_) => 1004,
                    
                    // Terminal operations (1003-1006)
                    Mt5Error::InitializeTerminal(_) => 1003,
                    Mt5Error::GetTerminalInfo(_) => 1004,
                    Mt5Error::GetSymbolList(_) => 1005,
                    Mt5Error::Ping { .. } => 1006,
                    
                    // Market data operations (1007)
                    Mt5Error::GetKlineData { .. } => 1007,
                    
                    // Trading operations (1008-1012)
                    Mt5Error::CreateOrder { .. } => 1008,
                    Mt5Error::GetOrder { .. } => 1009,
                    Mt5Error::GetPosition { .. } => 1010,
                    Mt5Error::GetDeal { .. } => 1011,
                    Mt5Error::GetPositionNumber { .. } => 1012,
                    
                    // Account operations (1013)
                    Mt5Error::GetAccountInfo { .. } => 1013,
                    
                    // WebSocket errors (1014)
                    Mt5Error::WebSocket { .. } => 1014,
                    
                    // Connection and initialization errors (1015-1017)
                    Mt5Error::Connection { .. } => 1015,
                    Mt5Error::Initialization(_) => 1016,
                    Mt5Error::Configuration(_) => 1017,
                    
                    // Server and service errors (1018-1021)
                    Mt5Error::Server(_) => 1018,
                    Mt5Error::Timeout(_) => 1019,
                    Mt5Error::Authentication(_) => 1020,
                    Mt5Error::Validation(_) => 1021,
                    
                    // Internal errors (1022)
                    Mt5Error::Internal(_) => 1022,
                    
                    // This should never happen due to outer match, but needed for completeness
                    Mt5Error::DataProcessor(_) => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    pub fn http(message: impl Into<String>, terminal_id: i32, port: u16, source: reqwest::Error) -> Self {
        Self::Http { message: message.into(), terminal_id, port, source }
    }

    pub fn http_client_not_created(terminal_id: i32, port: u16) -> Self {
        Self::HttpClientNotCreated { terminal_id, port }
    }

    pub fn ping(message: impl Into<String>, terminal_id: i32, port: u16, source: Option<reqwest::Error>) -> Self {
        Self::Ping { message: message.into(), terminal_id, port, source }
    }

    pub fn initialize_terminal(message: impl Into<String>) -> Self {
        Self::InitializeTerminal(message.into())
    }

    pub fn get_terminal_info(message: impl Into<String>) -> Self {
        Self::GetTerminalInfo(message.into())
    }

    pub fn get_symbol_list(message: impl Into<String>) -> Self {
        Self::GetSymbolList(message.into())
    }

    pub fn get_kline_data(symbol: impl Into<String>, message: impl Into<String>, code: Option<MT5ErrorCode>) -> Self {
        Self::GetKlineData {
            symbol: symbol.into(),
            message: message.into(),
            code,
        }
    }
    
    pub fn create_order(symbol: impl Into<String>, message: impl Into<String>, code: Option<MT5ErrorCode>) -> Self {
        Self::CreateOrder {
            symbol: symbol.into(),
            message: message.into(),
            code,
        }
    }
    
    pub fn get_order(order_id: i64, message: impl Into<String>) -> Self {
        Self::GetOrder {
            order_id,
            message: message.into(),
        }
    }
    
    pub fn get_position(position_id: i64, message: impl Into<String>) -> Self {
        Self::GetPosition {
            position_id,
            message: message.into(),
        }
    }
    
    pub fn get_deal_by_deal_id(deal_id: i64, message: impl Into<String>) -> Self {
        Self::GetDeal {
            message: message.into(),
            deal_id: Some(deal_id),
            position_id: None,
            order_id: None,
        }
    }
    
    pub fn get_deal_by_position_id(position_id: i64, message: impl Into<String>) -> Self {
        Self::GetDeal {
            message: message.into(),
            deal_id: None,
            position_id: Some(position_id),
            order_id: None,
        }
    }
    
    pub fn get_deal_by_order_id(order_id: i64, message: impl Into<String>) -> Self {
        Self::GetDeal {
            message: message.into(),
            deal_id: None,
            position_id: None,
            order_id: Some(order_id),
        }
    }
    
    pub fn get_position_number(symbol: impl Into<String>, message: impl Into<String>) -> Self {
        Self::GetPositionNumber {
            symbol: symbol.into(),
            message: message.into(),
        }
    }

    pub fn get_account_info(message: impl Into<String>, terminal_id: i32, port: u16) -> Self {
        Self::GetAccountInfo { message: message.into(), terminal_id, port }
    }

    pub fn websocket<S: Into<String>>(message: S, account_id: AccountId, url: String, source: tokio_tungstenite::tungstenite::error::Error) -> Self {
        Self::WebSocket {
            message: message.into(),
            account_id,
            url,
            source,
        }
    }
    
    pub fn data_processor(error: DataProcessorError) -> Self {
        Self::DataProcessor(error)
    }
    
    pub fn connection<S: Into<String>>(message: S, terminal_id: i32, port: u16) -> Self {
        Self::Connection { message: message.into(), terminal_id, port }
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
            Mt5Error::DataProcessor(data_err) => data_err.get_prefix(),
            _ => self.get_prefix(),
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }

    fn context(&self) -> Vec<(&'static str, String)> {
        match self {
            Mt5Error::GetKlineData { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5Error::CreateOrder { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5Error::GetOrder { order_id, .. } => {
                vec![("order_id", order_id.to_string())]
            },
            Mt5Error::GetPosition { position_id, .. } => {
                vec![("position_id", position_id.to_string())]
            },
            Mt5Error::GetPositionNumber { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5Error::GetDeal { deal_id, position_id, order_id, .. } => {
                let mut ctx = vec![];
                if let Some(id) = deal_id {
                    ctx.push(("deal_id", id.to_string()));
                }
                if let Some(id) = position_id {
                    ctx.push(("position_id", id.to_string()));
                }
                if let Some(id) = order_id {
                    ctx.push(("order_id", id.to_string()));
                }
                ctx
            },
            Mt5Error::WebSocket { account_id, url, .. } => {
                vec![
                    ("account_id", account_id.to_string()),
                    ("url", url.clone())
                ]
            },
            Mt5Error::Connection { terminal_id, port, .. } => {
                vec![
                    ("terminal_id", terminal_id.to_string()),
                    ("port", port.to_string())
                ]
            },
            _ => vec![],
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