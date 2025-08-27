use thiserror::Error;
use crate::error::ErrorCode;

pub type MT5ErrorCode = i64;

#[derive(Error, Debug)]
pub enum Mt5HttpClientError {
    #[error("http request failed")]
    Http(#[from] reqwest::Error),
    
    #[error("json parsing failed")]
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
    
    #[error("failed to get account info: {0}")]
    GetAccountInfo(String),
    
    #[error("server ping failed: {0}")]
    Ping(String),
    
    #[error("MT5 HTTP client internal error: {0}")]
    Internal(String),
}

impl Mt5HttpClientError {
    /// Returns the error prefix for MT5 HTTP client errors
    pub fn get_prefix(&self) -> &'static str {
        "MT5_HTTP_CLIENT"
    }
    
    /// Returns a string error code for MT5 HTTP client errors (format: MT5_HTTP_CLIENT_NNNN)
    pub fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1002)
            Mt5HttpClientError::Http(_) => 1001,
            Mt5HttpClientError::Json(_) => 1002,
            
            // Terminal operations (1003-1006)
            Mt5HttpClientError::InitializeTerminal(_) => 1003,
            Mt5HttpClientError::GetTerminalInfo(_) => 1004,
            Mt5HttpClientError::GetSymbolList(_) => 1005,
            Mt5HttpClientError::Ping(_) => 1006,
            
            // Market data operations (1007)
            Mt5HttpClientError::GetKlineData { .. } => 1007,
            
            // Trading operations (1008-1012)
            Mt5HttpClientError::CreateOrder { .. } => 1008,
            Mt5HttpClientError::GetOrder { .. } => 1009,
            Mt5HttpClientError::GetPosition { .. } => 1010,
            Mt5HttpClientError::GetDeal { .. } => 1011,
            Mt5HttpClientError::GetPositionNumber { .. } => 1012,
            
            // Account operations (1013)
            Mt5HttpClientError::GetAccountInfo(_) => 1013,
            
            // Internal errors (1014)
            Mt5HttpClientError::Internal(_) => 1014,
        };
        format!("{}_{:04}", prefix, code)
    }

    pub fn ping(message: impl Into<String>) -> Self {
        Self::Ping(message.into())
    }

    pub fn initialize_terminal(message: impl Into<String>) -> Self {
        Self::InitializeTerminal(message.into())
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
    
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

// Implement the StarRiverErrorTrait for Mt5HttpClientError  
impl crate::error::error_trait::StarRiverErrorTrait for Mt5HttpClientError {
    fn get_prefix(&self) -> &'static str {
        self.get_prefix()
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn category(&self) -> &'static str {
        "mt5_http"
    }
    
    fn is_retriable(&self) -> bool {
        matches!(self,
            Mt5HttpClientError::Http(_) |
            Mt5HttpClientError::Ping(_)
        )
    }
    
    fn is_client_error(&self) -> bool {
        matches!(self,
            Mt5HttpClientError::Json(_)
        )
    }
    
    fn message(&self) -> &str {
        match self {
            Mt5HttpClientError::InitializeTerminal(msg) |
            Mt5HttpClientError::GetTerminalInfo(msg) |
            Mt5HttpClientError::GetSymbolList(msg) |
            Mt5HttpClientError::GetAccountInfo(msg) |
            Mt5HttpClientError::Ping(msg) |
            Mt5HttpClientError::Internal(msg) => msg,
            Mt5HttpClientError::GetKlineData { message, .. } |
            Mt5HttpClientError::CreateOrder { message, .. } |
            Mt5HttpClientError::GetOrder { message, .. } |
            Mt5HttpClientError::GetPosition { message, .. } |
            Mt5HttpClientError::GetDeal { message, .. } |
            Mt5HttpClientError::GetPositionNumber { message, .. } => message,
            Mt5HttpClientError::Http(_) => "http request failed",
            Mt5HttpClientError::Json(_) => "json parsing failed",
        }
    }
    
    fn context(&self) -> Vec<(&'static str, String)> {
        match self {
            Mt5HttpClientError::GetKlineData { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5HttpClientError::CreateOrder { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5HttpClientError::GetOrder { order_id, .. } => {
                vec![("order_id", order_id.to_string())]
            },
            Mt5HttpClientError::GetPosition { position_id, .. } => {
                vec![("position_id", position_id.to_string())]
            },
            Mt5HttpClientError::GetPositionNumber { symbol, .. } => {
                vec![("symbol", symbol.clone())]
            },
            Mt5HttpClientError::GetDeal { deal_id, position_id, order_id, .. } => {
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
            _ => vec![],
        }
    }
}

// Implement ErrorContext trait for Mt5HttpClientError
impl<T> crate::error::error_trait::ErrorContext<T, Mt5HttpClientError> for Result<T, Mt5HttpClientError> {
    fn with_context<F>(self, f: F) -> Result<T, Mt5HttpClientError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            Mt5HttpClientError::Internal(format!("{}: {}", context, e))
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, Mt5HttpClientError> {
        self.map_err(|e| {
            Mt5HttpClientError::Internal(format!("MT5 HTTP Operation '{}': {}", operation, e))
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, Mt5HttpClientError> {
        self.map_err(|e| {
            Mt5HttpClientError::Internal(format!("MT5 HTTP {} '{}': {}", resource_type, resource_id, e))
        })
    }
}