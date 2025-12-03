use std::collections::HashMap;

use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::AccountId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
};

use super::data_processor_error::Mt5DataProcessorError;

pub type MT5ErrorCode = i64;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Mt5Error {
    #[snafu(transparent)]
    DataProcessorError {
        source: Mt5DataProcessorError,
        backtrace: Backtrace,
    },

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
    Response {
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
    HttpClientNotCreated { terminal_id: i32, port: u16, backtrace: Backtrace },

    #[snafu(display("json parsing error"))]
    Json { source: serde_json::Error, backtrace: Backtrace },

    #[snafu(display("failed to initialize terminal: {message}"))]
    InitializeTerminalFailed {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("terminal {terminal_id} not initialized: port={port}"))]
    TerminalNotInitialized { terminal_id: i32, port: u16, backtrace: Backtrace },

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

    #[snafu(display("get '{symbol}' info failed. terminal id is {terminal_id}, port is {port}"))]
    GetSymbolInfo {
        message: String,
        symbol: String,
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
    Retcode { terminal_id: i32, port: u16, backtrace: Backtrace },

    #[snafu(display("Failed to get order id: terminal_id={terminal_id}, port={port}"))]
    OrderId { terminal_id: i32, port: u16, backtrace: Backtrace },

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

    #[snafu(display("MetaTrader5 connection error: message={message}, terminal_id={terminal_id}, port={port}"))]
    Connection {
        message: String,
        terminal_id: i32,
        port: u16,
        backtrace: Backtrace,
    },

    #[snafu(display("MetaTrader5 initialization error: {message}"))]
    Initialization { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 configuration error: {message}"))]
    Configuration { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 timeout error: {message}"))]
    Timeout { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 authentication error: {message}"))]
    Authentication { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 validation error: {message}"))]
    Validation { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 other error: {message}"))]
    Other { message: String, backtrace: Backtrace },

    #[snafu(display("MetaTrader5 http client port not set: terminal_id:{terminal_id}"))]
    HttpClientPortNotSet { terminal_id: i32, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for Mt5Error {
    fn get_prefix(&self) -> &'static str {
        "MT5"
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            Mt5Error::DataProcessorError { source, .. } => source.error_code(),

            // For direct MT5 errors, use MT5 prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // HTTP and JSON errors (1001-1004)
                    Mt5Error::Network { .. } => 1001,                  //网络错误
                    Mt5Error::Server { .. } => 1002,                   //服务器错误
                    Mt5Error::NoSuccessFieldInResponse { .. } => 1002, //响应中缺少成功字段
                    Mt5Error::HttpClientNotCreated { .. } => 1003,     //HTTP客户端未创建
                    Mt5Error::Json { .. } => 1004,                     //JSON解析错误
                    Mt5Error::Response { .. } => 1005,                 //响应错误

                    // Terminal operations (1005-1008)
                    Mt5Error::InitializeTerminalFailed { .. } => 1005, //初始化终端失败
                    Mt5Error::GetTerminalInfo { .. } => 1006,          //获取终端信息
                    Mt5Error::GetSymbolList { .. } => 1007,            //获取交易品种列表
                    Mt5Error::GetSymbolInfo { .. } => 1008,            //获取交易品种信息
                    Mt5Error::Ping { .. } => 1009,                     //Ping

                    // Market data operations (1010)
                    Mt5Error::GetKlineData { .. } => 1010, //获取K线数据

                    // Trading operations (1010-1014)
                    Mt5Error::CreateOrder { .. } => 1011,         //创建订单
                    Mt5Error::GetOrder { .. } => 1012,            //获取订单
                    Mt5Error::GetPosition { .. } => 1013,         //获取持仓
                    Mt5Error::GetDealByPositionId { .. } => 1014, //通过持仓ID获取交易
                    Mt5Error::GetDeal { .. } => 1015,             //获取交易
                    Mt5Error::GetDealByDealId { .. } => 1016,     //通过交易ID获取交易
                    Mt5Error::GetDealByOrderId { .. } => 1017,    //通过订单ID获取交易
                    Mt5Error::GetPositionNumber { .. } => 1018,   //获取持仓数量

                    // Account operations (1018)
                    Mt5Error::GetAccountInfo { .. } => 1019, //获取账户信息
                    Mt5Error::Retcode { .. } => 1020,        //获取返回码

                    // Order operations (1020)
                    Mt5Error::OrderId { .. } => 1021, //获取订单ID

                    // WebSocket errors (1016)
                    Mt5Error::WebSocket { .. } => 1022, //WebSocket错误

                    // Connection and initialization errors (1017-1019)
                    Mt5Error::Connection { .. } => 1023,     //连接错误
                    Mt5Error::Initialization { .. } => 1024, //初始化错误
                    Mt5Error::TerminalNotInitialized { .. } => 1025,
                    Mt5Error::Configuration { .. } => 1024,

                    // Server and service errors (1020-1022)
                    Mt5Error::Timeout { .. } => 1026,
                    Mt5Error::Authentication { .. } => 1021,
                    Mt5Error::Validation { .. } => 1027,

                    // Internal errors (1023)
                    Mt5Error::Other { .. } => 1028,

                    Mt5Error::HttpClientPortNotSet { .. } => 1029,
                    _ => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                Mt5Error::Network { terminal_id, url, .. } => {
                    format!("网络错误: 终端ID={}, URL={}", terminal_id, url)
                }
                Mt5Error::Server {
                    terminal_id,
                    url,
                    status_code,
                    ..
                } => {
                    format!("服务器错误: 终端ID={}, URL={}, 状态码={}", terminal_id, url, status_code)
                }
                Mt5Error::Response { terminal_id, url, .. } => {
                    format!("响应错误: 终端ID={}, URL={}", terminal_id, url)
                }
                Mt5Error::NoSuccessFieldInResponse { terminal_id, url, .. } => {
                    format!("响应中缺少成功字段: 终端ID={}, URL={}", terminal_id, url)
                }
                Mt5Error::HttpClientNotCreated { terminal_id, port, .. } => {
                    format!("HTTP客户端未创建: 终端ID={}, 端口={}", terminal_id, port)
                }
                Mt5Error::Json { .. } => "JSON解析错误".to_string(),
                Mt5Error::InitializeTerminalFailed {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("终端初始化失败: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::TerminalNotInitialized { terminal_id, port, .. } => {
                    format!("终端 {} 未初始化: 端口={}", terminal_id, port)
                }
                Mt5Error::GetTerminalInfo {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("获取终端信息失败: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::GetSymbolList {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("获取交易品种列表失败: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::GetSymbolInfo {
                    symbol,
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!(
                        "获取 '{}' 交易品种信息失败: {}, 终端ID 是{}, 端口是{}",
                        symbol, message, terminal_id, port
                    )
                }
                Mt5Error::GetKlineData { symbol, message, .. } => {
                    format!("获取 '{}' K线数据失败: {}", symbol, message)
                }
                Mt5Error::CreateOrder { symbol, message, .. } => {
                    format!("为 '{}' 创建订单失败: {}", symbol, message)
                }
                Mt5Error::GetOrder { order_id, message, .. } => {
                    format!("获取订单 {} 失败: {}", order_id, message)
                }
                Mt5Error::GetPosition { position_id, message, .. } => {
                    format!("获取持仓 {} 失败: {}", position_id, message)
                }
                Mt5Error::GetDealByPositionId { position_id, message, .. } => {
                    format!("通过持仓ID {} 获取交易失败: {}", position_id, message)
                }
                Mt5Error::GetDeal {
                    message,
                    deal_id,
                    position_id,
                    order_id,
                    ..
                } => {
                    let mut details = Vec::new();
                    if let Some(id) = deal_id {
                        details.push(format!("交易ID={}", id));
                    }
                    if let Some(id) = position_id {
                        details.push(format!("持仓ID={}", id));
                    }
                    if let Some(id) = order_id {
                        details.push(format!("订单ID={}", id));
                    }
                    let detail_str = if details.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", details.join(", "))
                    };
                    format!("获取交易失败: {}{}", message, detail_str)
                }
                Mt5Error::GetDealByDealId { deal_id, message, .. } => {
                    format!("通过交易ID {} 获取交易失败: {}", deal_id, message)
                }
                Mt5Error::GetDealByOrderId { order_id, message, .. } => {
                    format!("通过订单ID {} 获取交易失败: {}", order_id, message)
                }
                Mt5Error::GetPositionNumber { symbol, message, .. } => {
                    format!("获取 '{}' 持仓数量失败: {}", symbol, message)
                }
                Mt5Error::GetAccountInfo {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("获取账户信息失败: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::Retcode { terminal_id, port, .. } => {
                    format!("获取返回码失败: 终端ID={}, 端口={}", terminal_id, port)
                }
                Mt5Error::OrderId { terminal_id, port, .. } => {
                    format!("获取订单ID失败: 终端ID={}, 端口={}", terminal_id, port)
                }
                Mt5Error::Ping {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("Ping失败: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::WebSocket {
                    message, account_id, url, ..
                } => {
                    format!("MetaTrader5 WebSocket错误: {}, 账户ID: {}, URL: {}", message, account_id, url)
                }
                Mt5Error::DataProcessorError { source, .. } => {
                    format!("数据处理器错误: {}", source.error_message(language))
                }
                Mt5Error::Connection {
                    message,
                    terminal_id,
                    port,
                    ..
                } => {
                    format!("MetaTrader5 连接错误: {}, 终端ID={}, 端口={}", message, terminal_id, port)
                }
                Mt5Error::Initialization { message, .. } => {
                    format!("MetaTrader5 初始化错误: {}", message)
                }
                Mt5Error::Configuration { message, .. } => {
                    format!("MetaTrader5 配置错误: {}", message)
                }
                Mt5Error::Timeout { message, .. } => {
                    format!("MetaTrader5 超时错误: {}", message)
                }
                Mt5Error::Authentication { message, .. } => {
                    format!("MetaTrader5 认证错误: {}", message)
                }
                Mt5Error::Validation { message, .. } => {
                    format!("MetaTrader5 验证错误: {}", message)
                }
                Mt5Error::Other { message, .. } => {
                    format!("MetaTrader5 其他错误: {}", message)
                }

                Mt5Error::HttpClientPortNotSet { terminal_id, .. } => {
                    format!("MetaTrader5 HTTP客户端端口未设置: 终端ID={}", terminal_id)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // Errors with source that implement our trait
            Mt5Error::DataProcessorError { source, .. } => generate_error_code_chain(source, self.error_code()),

            // Errors with source that don't implement our trait - start chain here
            Mt5Error::Network { .. } | Mt5Error::Response { .. } | Mt5Error::Json { .. } | Mt5Error::WebSocket { .. } => {
                vec![self.error_code()]
            }

            // Errors without source - return own error code
            _ => vec![self.error_code()],
        }
    }
}
