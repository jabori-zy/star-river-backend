use thiserror::Error;

pub type MT5ErrorCode = i64;

#[derive(Error, Debug)]
pub enum Mt5HttpClientError {
    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON parsing failed")]
    Json(#[from] serde_json::Error),
    
    #[error("Failed to initialize terminal: {0}")]
    InitializeTerminal(String),
    
    #[error("Failed to get terminal info: {0}")]
    GetTerminalInfo(String),
    
    #[error("Failed to get kline data for symbol '{symbol}': {message}")]
    GetKlineData { 
        symbol: String, 
        message: String, 
        code: Option<MT5ErrorCode> 
    },
    
    #[error("Failed to create order for symbol '{symbol}': {message}")]
    CreateOrder { 
        symbol: String, 
        message: String,
        code: Option<MT5ErrorCode>
    },
    
    #[error("Failed to get order {order_id}: {message}")]
    GetOrder { 
        order_id: i64, 
        message: String 
    },
    
    #[error("Failed to get position {position_id}: {message}")]
    GetPosition { 
        position_id: i64, 
        message: String 
    },
    
    #[error("Failed to get deal: {message}")]
    GetDeal {
        message: String,
        deal_id: Option<i64>,
        position_id: Option<i64>,
        order_id: Option<i64>,
    },
    
    #[error("Failed to get position number for symbol '{symbol}': {message}")]
    GetPositionNumber { 
        symbol: String, 
        message: String 
    },
    
    #[error("Failed to get account info: {0}")]
    GetAccountInfo(String),
    
    #[error("Server ping failed: {0}")]
    Ping(String),
}

impl Mt5HttpClientError {
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
}