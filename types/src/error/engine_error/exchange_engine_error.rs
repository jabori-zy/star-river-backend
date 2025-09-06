use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::exchange_client_error::ExchangeClientError;
use crate::error::exchange_client_error::Mt5Error;
use crate::custom_type::AccountId;
use crate::market::Exchange;
use sea_orm::error::DbErr;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeEngineError {
    // === Registration & Configuration Errors ===
    #[snafu(display("exchange registration failed for account {account_id}: {message}"))]
    RegisterExchangeFailed {
        message: String,
        account_id: AccountId,
        exchange_type: Exchange,
        source: ExchangeClientError,
        backtrace: Backtrace,
    },
    
    #[snafu(display("exchange unregistration failed for account {account_id}: {message}"))]
    UnregistrationFailed {
        message: String,
        account_id: AccountId,
        exchange_type: Exchange,
        backtrace: Backtrace,
    },
    
    #[snafu(transparent)]
    Database {
        source: DbErr,
        backtrace: Backtrace,
    },
    
    #[snafu(display("account {account_id}'s exchange type {:?} is unsupported", exchange_type))]
    UnsupportedExchangeType {
        exchange_type: Exchange,
        account_id: AccountId,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    Mt5 {
        source: Mt5Error,
    },
    
    // === Exchange Client Management Errors ===
    #[snafu(display("exchange client not found for account {account_id}"))]
    ExchangeClientNotFound {
        message: String,
        account_id: AccountId,
        requested_operation: Option<String>,
        backtrace: Backtrace,
    },
    
    #[snafu(display("exchange client operation failed for account {account_id}: {message}"))]
    ExchangeClientOperationFailed {
        message: String,
        account_id: AccountId,
        operation: String,
        exchange_type: Exchange,
        backtrace: Backtrace,
    },
    
    #[snafu(display("exchange client type conversion failed for account {account_id}: {message}"))]
    ExchangeClientTypeConversionFailed {
        message: String,
        account_id: AccountId,
        expected_type: String,
        actual_type: String,
        backtrace: Backtrace,
    },
    
    // === Database Errors ===
    #[snafu(display("database operation failed: {message}"))]
    DatabaseOperationFailed {
        message: String,
        operation: String,
        account_id: Option<AccountId>,
        table: Option<String>,
        backtrace: Backtrace,
    },
    
    #[snafu(display("database connection failed: {message}"))]
    DatabaseConnectionFailed {
        message: String,
        database_url: Option<String>,
        backtrace: Backtrace,
    },
    
    // === Timeout Errors ===
    #[snafu(display("operation timeout for account {}: {message}", account_id.map(|id| id.to_string()).unwrap_or("unknown".to_string())))]
    OperationTimeout {
        message: String,
        account_id: Option<AccountId>,
        operation: String,
        timeout_duration: String,
        retry_count: Option<u32>,
        backtrace: Backtrace,
    },
    
    // === Configuration Errors ===
    #[snafu(display("configuration error: {message}"))]
    ConfigurationError {
        message: String,
        config_key: Option<String>,
        account_id: Option<AccountId>,
        backtrace: Backtrace,
    },
    
    #[snafu(display("environment error: {message}"))]
    EnvironmentError {
        message: String,
        variable: Option<String>,
        expected: Option<String>,
        backtrace: Backtrace,
    },
    
    // === Event & Command Handling Errors ===
    #[snafu(display("event publishing failed: {message}"))]
    EventPublishingFailed {
        message: String,
        account_id: Option<AccountId>,
        event_type: Option<String>,
        backtrace: Backtrace,
    },
    
    #[snafu(display("command handling failed: {message}"))]
    CommandHandlingFailed {
        message: String,
        account_id: Option<AccountId>,
        command_type: String,
        backtrace: Backtrace,
    },
    
    #[snafu(transparent)]
    ExchangeClientError {
        source: ExchangeClientError,
    },
    
    // === Generic Errors ===
    #[snafu(display("internal exchange engine error: {message}"))]
    Internal {
        message: String,
        component: Option<String>,
        context: Option<String>,
        account_id: Option<AccountId>,
        backtrace: Backtrace,
    },
    
    #[snafu(display("feature not implemented: {message}"))]
    NotImplemented {
        message: String,
        feature: String,
        exchange_type: Exchange,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for ExchangeEngineError
impl crate::error::error_trait::StarRiverErrorTrait for ExchangeEngineError {
    fn get_prefix(&self) -> &'static str {
        match self {
            // For nested errors, delegate to the inner error's prefix
            ExchangeEngineError::ExchangeClientError { source } => source.get_prefix(),
            ExchangeEngineError::Mt5 { source } => source.get_prefix(),
            _ => "EXCHANGE_ENGINE",
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeEngineError::ExchangeClientError { source } => source.error_code(),
            ExchangeEngineError::Mt5 { source } => source.error_code(),
            
            // For direct exchange engine errors, use EXCHANGE_ENGINE prefix
            _ => {
                let prefix = "EXCHANGE_ENGINE";
                let code = match self {
                    // Registration & Configuration (1001-1004)
                    ExchangeEngineError::RegisterExchangeFailed { .. } => 1001,
                    ExchangeEngineError::UnregistrationFailed { .. } => 1002,
                    ExchangeEngineError::Database { .. } => 1003,
                    ExchangeEngineError::UnsupportedExchangeType { .. } => 1004,
                    
                    // Exchange Client Management (1011-1013)
                    ExchangeEngineError::ExchangeClientNotFound { .. } => 1011,
                    ExchangeEngineError::ExchangeClientOperationFailed { .. } => 1012,
                    ExchangeEngineError::ExchangeClientTypeConversionFailed { .. } => 1013,
                    
                    // Database (1014-1015)
                    ExchangeEngineError::DatabaseOperationFailed { .. } => 1014,
                    ExchangeEngineError::DatabaseConnectionFailed { .. } => 1015,
                    
                    // Timeout (1016)
                    ExchangeEngineError::OperationTimeout { .. } => 1016,
                    
                    // Configuration (1017-1018)
                    ExchangeEngineError::ConfigurationError { .. } => 1017,
                    ExchangeEngineError::EnvironmentError { .. } => 1018,
                    
                    // Event & Command (1019-1020)
                    ExchangeEngineError::EventPublishingFailed { .. } => 1019,
                    ExchangeEngineError::CommandHandlingFailed { .. } => 1020,
                    
                    // Generic (1021-1022)
                    ExchangeEngineError::Internal { .. } => 1021,
                    ExchangeEngineError::NotImplemented { .. } => 1022,
                    
                    // This should never happen due to outer match, but needed for completeness
                    ExchangeEngineError::ExchangeClientError { .. } |
                    ExchangeEngineError::Mt5 { .. } => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }
    
    fn context(&self) -> HashMap<&'static str, String> {
        let mut ctx = HashMap::new();
        match self {
            // For nested errors, delegate to the inner error's context
            ExchangeEngineError::ExchangeClientError { source } => return source.context(),
            ExchangeEngineError::Mt5 { source } => return source.context(),
            
            ExchangeEngineError::RegisterExchangeFailed { account_id, exchange_type, .. } => {
                ctx.insert("account_id", account_id.to_string());
                ctx.insert("exchange_type", format!("{:?}", exchange_type));
            },
            
            ExchangeEngineError::ExchangeClientOperationFailed { account_id, operation, exchange_type, .. } => {
                ctx.insert("account_id", account_id.to_string());
                ctx.insert("operation", operation.clone());
                ctx.insert("exchange_type", format!("{:?}", exchange_type));
            },
            ExchangeEngineError::DatabaseOperationFailed { operation, account_id, table, .. } => {
                ctx.insert("operation", operation.clone());
                if let Some(acc_id) = account_id {
                    ctx.insert("account_id", acc_id.to_string());
                }
                if let Some(t) = table {
                    ctx.insert("table", t.clone());
                }
            },
            ExchangeEngineError::OperationTimeout { account_id, operation, timeout_duration, retry_count, .. } => {
                ctx.insert("operation", operation.clone());
                ctx.insert("timeout_duration", timeout_duration.clone());
                if let Some(acc_id) = account_id {
                    ctx.insert("account_id", acc_id.to_string());
                }
                if let Some(retry) = retry_count {
                    ctx.insert("retry_count", retry.to_string());
                }
            },
            ExchangeEngineError::EventPublishingFailed { account_id, event_type, .. } => {
                if let Some(acc_id) = account_id {
                    ctx.insert("account_id", acc_id.to_string());
                }
                if let Some(ev_type) = event_type {
                    ctx.insert("event_type", ev_type.clone());
                }
            },
            ExchangeEngineError::CommandHandlingFailed { account_id, command_type, .. } => {
                ctx.insert("command_type", command_type.clone());
                if let Some(acc_id) = account_id {
                    ctx.insert("account_id", acc_id.to_string());
                }
            },
            _ => {
                // For simpler error types, extract account_id if available
                match self {
                    ExchangeEngineError::UnregistrationFailed { account_id, .. } |
                    ExchangeEngineError::ExchangeClientNotFound { account_id, .. } => {
                        ctx.insert("account_id", account_id.to_string());
                    },
                    _ => {},
                }
            }
        }
        ctx
    }

    fn is_recoverable(&self) -> bool {
        match self {
            // For nested errors, delegate to the inner error's recoverability
            ExchangeEngineError::ExchangeClientError { source } => source.is_recoverable(),
            ExchangeEngineError::Mt5 { source } => source.is_recoverable(),
            
            // Recoverable errors (network, connection, temporary issues, operations)
            _ => matches!(self,
                // Database and network-related errors may be recoverable
                ExchangeEngineError::Database { .. } |
                ExchangeEngineError::DatabaseOperationFailed { .. } |
                ExchangeEngineError::DatabaseConnectionFailed { .. } |
                
                // Exchange client operations may be recoverable
                ExchangeEngineError::RegisterExchangeFailed { .. } |
                ExchangeEngineError::ExchangeClientOperationFailed { .. } |
                ExchangeEngineError::ExchangeClientNotFound { .. } |
                
                // Timeout errors are usually recoverable
                ExchangeEngineError::OperationTimeout { .. } |
                
                // Event and command handling failures may be recoverable
                ExchangeEngineError::EventPublishingFailed { .. } |
                ExchangeEngineError::CommandHandlingFailed { .. }
            )
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            ExchangeEngineError::ExchangeClientError { source } => source.error_code_chain(),
            ExchangeEngineError::Mt5 { source } => source.error_code_chain(),
            ExchangeEngineError::Database { .. } => {
                // DbErr doesn't implement our trait, so just return our code
                vec![self.error_code()]
            },
            
            // For errors with source that implements our trait
            ExchangeEngineError::RegisterExchangeFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            
            // For errors without source or with external sources
            _ => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => {
                self.to_string()
            },
            Language::Chinese => {
                match self {
                    ExchangeEngineError::RegisterExchangeFailed { message, account_id, exchange_type, .. } => {
                        format!("账户 {} 注册交易所失败: {}, 交易所类型: {:?}", account_id, message, exchange_type)
                    },
                    ExchangeEngineError::UnregistrationFailed { message, account_id, exchange_type, .. } => {
                        format!("账户 {} 注销交易所失败: {}, 交易所类型: {:?}", account_id, message, exchange_type)
                    },
                    ExchangeEngineError::Database { source, .. } => {
                        format!("数据库错误: {}", source)
                    },
                    ExchangeEngineError::UnsupportedExchangeType { exchange_type, account_id, .. } => {
                        format!("账户 {} 的交易所类型 {:?} 不支持", account_id, exchange_type)
                    },
                    ExchangeEngineError::Mt5 { source } => {
                        format!("MetaTrader5错误: {}", source.get_error_message(language))
                    },
                    ExchangeEngineError::ExchangeClientNotFound { message, account_id, requested_operation, .. } => {
                        let op_str = if let Some(op) = requested_operation {
                            format!(", 请求操作: {}", op)
                        } else {
                            String::new()
                        };
                        format!("账户 {} 未找到交易所客户端: {}{}", account_id, message, op_str)
                    },
                    ExchangeEngineError::ExchangeClientOperationFailed { message, account_id, operation, exchange_type, .. } => {
                        format!("账户 {} 交易所客户端操作失败: {}, 操作: {}, 交易所: {:?}", account_id, message, operation, exchange_type)
                    },
                    ExchangeEngineError::ExchangeClientTypeConversionFailed { message, account_id, expected_type, actual_type, .. } => {
                        format!("账户 {} 交易所客户端类型转换失败: {}, 期望类型: {}, 实际类型: {}", account_id, message, expected_type, actual_type)
                    },
                    ExchangeEngineError::DatabaseOperationFailed { message, operation, account_id, table, .. } => {
                        let mut msg = format!("数据库操作失败: {}, 操作: {}", message, operation);
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户ID: {}", acc_id));
                        }
                        if let Some(t) = table {
                            msg.push_str(&format!(", 表: {}", t));
                        }
                        msg
                    },
                    ExchangeEngineError::DatabaseConnectionFailed { message, database_url, .. } => {
                        let url_str = if let Some(url) = database_url {
                            format!(", 数据库URL: {}", url)
                        } else {
                            String::new()
                        };
                        format!("数据库连接失败: {}{}", message, url_str)
                    },
                    ExchangeEngineError::OperationTimeout { message, account_id, operation, timeout_duration, retry_count, .. } => {
                        let mut msg = format!("操作超时: {}, 操作: {}, 超时时长: {}", message, operation, timeout_duration);
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户: {}", acc_id));
                        }
                        if let Some(retry) = retry_count {
                            msg.push_str(&format!(", 重试次数: {}", retry));
                        }
                        msg
                    },
                    ExchangeEngineError::ConfigurationError { message, config_key, account_id, .. } => {
                        let mut msg = format!("配置错误: {}", message);
                        if let Some(key) = config_key {
                            msg.push_str(&format!(", 配置键: {}", key));
                        }
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户ID: {}", acc_id));
                        }
                        msg
                    },
                    ExchangeEngineError::EnvironmentError { message, variable, expected, .. } => {
                        let mut msg = format!("环境错误: {}", message);
                        if let Some(var) = variable {
                            msg.push_str(&format!(", 变量: {}", var));
                        }
                        if let Some(exp) = expected {
                            msg.push_str(&format!(", 期望值: {}", exp));
                        }
                        msg
                    },
                    ExchangeEngineError::EventPublishingFailed { message, account_id, event_type, .. } => {
                        let mut msg = format!("事件发布失败: {}", message);
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户ID: {}", acc_id));
                        }
                        if let Some(ev_type) = event_type {
                            msg.push_str(&format!(", 事件类型: {}", ev_type));
                        }
                        msg
                    },
                    ExchangeEngineError::CommandHandlingFailed { message, account_id, command_type, .. } => {
                        let mut msg = format!("命令处理失败: {}, 命令类型: {}", message, command_type);
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户ID: {}", acc_id));
                        }
                        msg
                    },
                    ExchangeEngineError::ExchangeClientError { source } => {
                        format!("交易所客户端错误: {}", source.get_error_message(language))
                    },
                    ExchangeEngineError::Internal { message, component, context, account_id, .. } => {
                        let mut msg = format!("交易所引擎内部错误: {}", message);
                        if let Some(comp) = component {
                            msg.push_str(&format!(", 组件: {}", comp));
                        }
                        if let Some(ctx) = context {
                            msg.push_str(&format!(", 上下文: {}", ctx));
                        }
                        if let Some(acc_id) = account_id {
                            msg.push_str(&format!(", 账户ID: {}", acc_id));
                        }
                        msg
                    },
                    ExchangeEngineError::NotImplemented { message, feature, exchange_type, .. } => {
                        format!("功能未实现: {}, 功能: {}, 交易所类型: {:?}", message, feature, exchange_type)
                    },
                }
            },
        }
    }
}