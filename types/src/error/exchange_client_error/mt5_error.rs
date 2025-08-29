use snafu::{Snafu, Backtrace, GenerateImplicitData};
use std::collections::HashMap;
use super::data_processor_error::DataProcessorError;
use crate::error::ErrorCode;
use crate::custom_type::AccountId;

pub type MT5ErrorCode = i64;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Mt5Error {

    #[snafu(display("network error: terminal_id={terminal_id}, url={url}"))]
    Network {
        terminal_id: i32,
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("server error: terminal_id={terminal_id}, url={url}"))]
    Server {
        terminal_id: i32,
        url: String,
        status_code: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("response error: terminal_id={terminal_id}, url={url}"))]
    Response{
        terminal_id: i32,
        url: String,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("no success field in the response"))]
    NoSuccessFieldInResponse {
        terminal_id: i32,
        url: String,
        backtrace: Backtrace,
    },

    #[snafu(display("http client not created: terminal_id={terminal_id}, port={port}"))]
    HttpClientNotCreated {
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("json parsing error"))]
    Json {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
    
    #[snafu(display("failed to initialize terminal: {message}"))]
    InitializeTerminal {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("terminal {terminal_id} not initialized: port={port}"))]
    TerminalNotInitialized {
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("failed to get terminal info: {message}"))]
    GetTerminalInfo {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("failed to get symbol list: {message}"))]
    GetSymbolList {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("failed to get kline data for symbol '{symbol}': {message}"))]
    GetKlineData { 
        symbol: String, 
        message: String, 
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("failed to create order for symbol '{symbol}': {message}"))]
    CreateOrder { 
        symbol: String, 
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("failed to get order {order_id}: {message}"))]
    GetOrder { 
        order_id: i64, 
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Failed to get position {position_id}: {message}"))]
    GetPosition {
        position_id: i64, 
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to get deal by position id {position_id}: {message}"))]
    GetDealByPositionId {
        position_id: i64,
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Failed to get deal: {message}"))]
    GetDeal {
        message: String,
        deal_id: Option<i64>,
        position_id: Option<i64>,
        order_id: Option<i64>,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to get deal by deal id {deal_id}: {message}"))]
    GetDealByDealId {
        deal_id: i64,
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to get deal by order id {order_id}: {message}"))]
    GetDealByOrderId {
        order_id: i64,
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Failed to get position number for symbol '{symbol}': {message}"))]
    GetPositionNumber {
        symbol: String, 
        message: String,
        code: Option<MT5ErrorCode>,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Failed to get account info: message={message}, terminal_id={terminal_id}, port={port}"))]
    GetAccountInfo {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to get retcode: terminal_id={terminal_id}, port={port}"))]
    Retcode {
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to get order id: terminal_id={terminal_id}, port={port}"))]
    OrderId {
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Ping failed: {message}, terminal_id={terminal_id}, port={port}"))]
    Ping {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    
    #[snafu(display("MetaTrader5 websocket error: {message}, account_id: {account_id}, url: {url}"))]
    WebSocket {
        message: String,
        account_id: AccountId,
        url: String,
        source: tokio_tungstenite::tungstenite::error::Error,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Data processor error"))]
    DataProcessor {
        source: DataProcessorError,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 connection error: message={message}, terminal_id={terminal_id}, port={port}"))]
    Connection {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 initialization error: {message}"))]
    Initialization {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 configuration error: {message}"))]
    Configuration {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 timeout error: {message}"))]
    Timeout {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 authentication error: {message}"))]
    Authentication {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 validation error: {message}"))]
    Validation {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("MetaTrader5 other error: {message}"))]
    Other {
        message: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for Mt5Error {
    fn get_prefix(&self) -> &'static str {
        "MT5"
    }
    
    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            Mt5Error::DataProcessor { source, .. } => source.error_code(),
            
            // For direct MT5 errors, use MT5 prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // HTTP and JSON errors (1001-1004)
                    Mt5Error::Network { .. } => 1001,
                    Mt5Error::Server { .. } => 1002,
                    Mt5Error::NoSuccessFieldInResponse { .. } => 1002,
                    Mt5Error::HttpClientNotCreated { .. } => 1003,
                    Mt5Error::Json { .. } => 1004,
                    Mt5Error::Response { .. } => 1005,
                    
                    // Terminal operations (1005-1008)
                    Mt5Error::InitializeTerminal { .. } => 1005,
                    Mt5Error::GetTerminalInfo { .. } => 1006,
                    Mt5Error::GetSymbolList { .. } => 1007,
                    Mt5Error::Ping { .. } => 1008,
                    
                    // Market data operations (1009)
                    Mt5Error::GetKlineData { .. } => 1009,
                    
                    // Trading operations (1010-1014)
                    Mt5Error::CreateOrder { .. } => 1010,
                    Mt5Error::GetOrder { .. } => 1011,
                    Mt5Error::GetPosition { .. } => 1012,
                    Mt5Error::GetDealByPositionId { .. } => 1013,
                    Mt5Error::GetDeal { .. } => 1014,
                    Mt5Error::GetDealByDealId { .. } => 1015,
                    Mt5Error::GetDealByOrderId { .. } => 1016,
                    Mt5Error::GetPositionNumber { .. } => 1017,

                    // Account operations (1018)
                    Mt5Error::GetAccountInfo { .. } => 1018,
                    Mt5Error::Retcode { .. } => 1019,
                    Mt5Error::OrderId { .. } => 1020,
                    
                    // WebSocket errors (1016)
                    Mt5Error::WebSocket { .. } => 1016,
                    
                    // Connection and initialization errors (1017-1019)
                    Mt5Error::Connection { .. } => 1017,
                    Mt5Error::Initialization { .. } => 1018,
                    Mt5Error::TerminalNotInitialized { .. } => 1018,
                    Mt5Error::Configuration { .. } => 1019,
                    
                    // Server and service errors (1020-1022)
                    Mt5Error::Timeout { .. } => 1020,
                    Mt5Error::Authentication { .. } => 1021,
                    Mt5Error::Validation { .. } => 1022,
                    
                    // Internal errors (1023)
                    Mt5Error::Other { .. } => 1023,

                    // This should never happen due to outer match, but needed for completeness
                    Mt5Error::DataProcessor { .. } => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let mut ctx = HashMap::new();
        match self {
            Mt5Error::DataProcessor { source, .. } => return source.context(),
            Mt5Error::GetKlineData { symbol, .. } => {
                ctx.insert("symbol", symbol.clone());
            },
            Mt5Error::CreateOrder { symbol, .. } => {
                ctx.insert("symbol", symbol.clone());
            },
            Mt5Error::GetOrder { order_id, .. } => {
                ctx.insert("order_id", order_id.to_string());
            },
            Mt5Error::GetPosition { position_id, .. } => {
                ctx.insert("position_id", position_id.to_string());
            },
            Mt5Error::GetPositionNumber { symbol, .. } => {
                ctx.insert("symbol", symbol.clone());
            },
            Mt5Error::GetDeal { deal_id, position_id, order_id, .. } => {
                if let Some(id) = deal_id {
                    ctx.insert("deal_id", id.to_string());
                }
                if let Some(id) = position_id {
                    ctx.insert("position_id", id.to_string());
                }
                if let Some(id) = order_id {
                    ctx.insert("order_id", id.to_string());
                }
            },
            Mt5Error::WebSocket { account_id, url, .. } => {
                ctx.insert("account_id", account_id.to_string());
                ctx.insert("url", url.clone());
            },
            Mt5Error::Connection { terminal_id, port, .. } => {
                ctx.insert("terminal_id", terminal_id.to_string());
                ctx.insert("port", port.to_string());
            },
            Mt5Error::HttpClientNotCreated { terminal_id, port, .. } => {
                ctx.insert("terminal_id", terminal_id.to_string());
                ctx.insert("port", port.to_string());
            },
            Mt5Error::GetAccountInfo { terminal_id, port, .. } => {
                ctx.insert("terminal_id", terminal_id.to_string());
                ctx.insert("port", port.to_string());
            },
            Mt5Error::Ping { terminal_id, port, .. } => {
                ctx.insert("terminal_id", terminal_id.to_string());
                ctx.insert("port", port.to_string());
            },
            _ => {},
        }
        ctx
    }

    fn is_recoverable(&self) -> bool {
        match self {
            // For nested errors, delegate to the inner error's recoverability
            Mt5Error::DataProcessor { source, .. } => source.is_recoverable(),
            
            // Server errors may be recoverable depending on status code
            Mt5Error::Server { status_code, .. } => {
                matches!(*status_code, 500..=599) // 5xx server errors are usually temporary
            },
            
            // Recoverable errors (network, connection, temporary issues, trading operations)
            _ => matches!(self,
                // Network-related errors are usually recoverable (temporary network issues)
                Mt5Error::Network { .. } |
                Mt5Error::Response { .. } |
                Mt5Error::Connection { .. } |
                Mt5Error::Timeout { .. } |
                Mt5Error::Ping { .. } |
                Mt5Error::WebSocket { .. } |
                
                // Terminal and initialization errors may be recoverable
                Mt5Error::InitializeTerminal { .. } |
                Mt5Error::GetTerminalInfo { .. } |
                Mt5Error::TerminalNotInitialized { .. } |
                Mt5Error::HttpClientNotCreated { .. } |
                Mt5Error::Initialization { .. } |
                
                // Trading operation errors may be recoverable (market conditions)
                Mt5Error::CreateOrder { .. } |
                Mt5Error::GetOrder { .. } |
                Mt5Error::GetPosition { .. } |
                Mt5Error::GetDealByPositionId { .. } |
                Mt5Error::GetDeal { .. } |
                Mt5Error::GetDealByDealId { .. } |
                Mt5Error::GetDealByOrderId { .. } |
                Mt5Error::GetPositionNumber { .. } |
                
                // Market data operations are usually recoverable
                Mt5Error::GetKlineData { .. } |
                Mt5Error::GetSymbolList { .. } |
                Mt5Error::GetAccountInfo { .. }
            )
        }
    }
}

// Convert from serde_json::Error automatically
impl From<serde_json::Error> for Mt5Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json {
            source: err,
            backtrace: Backtrace::generate(),
        }
    }
}

// Convert from DataProcessorError automatically
impl From<DataProcessorError> for Mt5Error {
    fn from(err: DataProcessorError) -> Self {
        Self::DataProcessor {
            source: err,
            backtrace: Backtrace::generate(),
        }
    }
}