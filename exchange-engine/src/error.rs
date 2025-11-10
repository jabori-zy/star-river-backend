//workspace crate
use engine_core::state_machine_error::EngineStateMachineError;
use exchange_client::{binance::error::BinanceError, exchange_error::ExchangeError, metatrader5::error::Mt5Error};
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::AccountId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
    exchange::Exchange,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeEngineError {
    // === Registration & Configuration Errors ===
    #[snafu(display("exchange registration failed for account {account_id}: {source}"))]
    RegisterExchangeFailed {
        account_id: AccountId,
        exchange_type: Exchange,
        source: ExchangeError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    EngineStateMachineError {
        source: EngineStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    BinanceError { source: BinanceError, backtrace: Backtrace },

    #[snafu(transparent)]
    Mt5Error { source: Mt5Error, backtrace: Backtrace },

    #[snafu(display("exchange unregistration failed for account {account_id}: {message}"))]
    UnregistrationFailed {
        message: String,
        account_id: AccountId,
        exchange_type: Exchange,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    Database { source: DbErr, backtrace: Backtrace },

    #[snafu(display("account {account_id}'s exchange type {:?} is unsupported", exchange_type))]
    UnsupportedExchangeType {
        exchange_type: Exchange,
        account_id: AccountId,
        backtrace: Backtrace,
    },

    // === Exchange Client Management Errors ===
    #[snafu(display("exchange {exchange_name} is not registered. account id is {account_id}"))]
    ExchangeClientNotRegistered {
        account_id: AccountId,
        exchange_name: String,
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
    ExchangeClientError { source: ExchangeError },

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
impl StarRiverErrorTrait for ExchangeEngineError {
    fn get_prefix(&self) -> &'static str {
        "EXCHANGE_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeEngineError::RegisterExchangeFailed { source, .. } => source.error_code(),
            ExchangeEngineError::ExchangeClientError { source } => source.error_code(),
            ExchangeEngineError::Mt5Error { source, .. } => source.error_code(),
            ExchangeEngineError::BinanceError { source, .. } => source.error_code(),
            ExchangeEngineError::EngineStateMachineError { source, .. } => source.error_code(),

            // For direct exchange engine errors, use EXCHANGE_ENGINE prefix
            _ => {
                let prefix = "EXCHANGE_ENGINE";
                let code = match self {
                    // Registration & Configuration (1002-1004)
                    ExchangeEngineError::UnregistrationFailed { .. } => 1002, // 注销交易所失败
                    ExchangeEngineError::BinanceError { .. } => 1003,         // 币安错误
                    ExchangeEngineError::Mt5Error { .. } => 1004,             // MetaTrader5错误
                    ExchangeEngineError::EngineStateMachineError { .. } => 1005, // 引擎状态机错误
                    ExchangeEngineError::Database { .. } => 1005,             // 数据库错误
                    ExchangeEngineError::UnsupportedExchangeType { .. } => 1006, // 不支持的交易所类型

                    // Exchange Client Management (1011-1013)
                    ExchangeEngineError::ExchangeClientNotRegistered { .. } => 1011, // 交易所客户端未找到
                    ExchangeEngineError::ExchangeClientOperationFailed { .. } => 1012, // 交易所客户端操作失败
                    ExchangeEngineError::ExchangeClientTypeConversionFailed { .. } => 1013, // 交易所客户端类型转换失败

                    // Database (1014-1015)
                    ExchangeEngineError::DatabaseOperationFailed { .. } => 1014, // 数据库操作失败
                    ExchangeEngineError::DatabaseConnectionFailed { .. } => 1015, // 数据库连接失败

                    // Timeout (1016)
                    ExchangeEngineError::OperationTimeout { .. } => 1016, // 操作超时

                    // Configuration (1017-1018)
                    ExchangeEngineError::ConfigurationError { .. } => 1017, // 配置错误
                    ExchangeEngineError::EnvironmentError { .. } => 1018,   // 环境错误

                    // Event & Command (1019-1020)
                    ExchangeEngineError::EventPublishingFailed { .. } => 1019, // 事件发布失败
                    ExchangeEngineError::CommandHandlingFailed { .. } => 1020, // 命令处理失败

                    // Generic (1021-1022)
                    ExchangeEngineError::Internal { .. } => 1021,       // 内部错误
                    ExchangeEngineError::NotImplemented { .. } => 1022, // 功能未实现

                    // This should never happen due to outer match, but needed for completeness
                    ExchangeEngineError::RegisterExchangeFailed { .. }
                    | ExchangeEngineError::ExchangeClientError { .. }
                    | ExchangeEngineError::Mt5Error { .. }
                    | ExchangeEngineError::BinanceError { .. }
                    | ExchangeEngineError::EngineStateMachineError { .. } => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            ExchangeEngineError::RegisterExchangeFailed { source, .. } => generate_error_code_chain(source),
            ExchangeEngineError::ExchangeClientError { source } => generate_error_code_chain(source),
            ExchangeEngineError::BinanceError { source, .. } => generate_error_code_chain(source),
            ExchangeEngineError::Mt5Error { source, .. } => generate_error_code_chain(source),
            ExchangeEngineError::EngineStateMachineError { source, .. } => generate_error_code_chain(source),
            ExchangeEngineError::Database { .. } => {
                // DbErr doesn't implement our trait, so just return our code
                vec![self.error_code()]
            }

            // For errors without source or with external sources
            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ExchangeEngineError::RegisterExchangeFailed {
                    account_id,
                    exchange_type,
                    source,
                    ..
                } => {
                    format!(
                        "账户 {} 注册交易所失败: 交易所类型: {:?}, 原因: {}",
                        account_id,
                        exchange_type,
                        source.error_message(language)
                    )
                }
                ExchangeEngineError::BinanceError { source, .. } => source.error_message(language),
                ExchangeEngineError::Mt5Error { source, .. } => source.error_message(language),
                ExchangeEngineError::EngineStateMachineError { source, .. } => source.error_message(language),
                ExchangeEngineError::UnregistrationFailed {
                    message,
                    account_id,
                    exchange_type,
                    ..
                } => {
                    format!("账户 {} 注销交易所失败: {}, 交易所类型: {:?}", account_id, message, exchange_type)
                }
                ExchangeEngineError::Database { source, .. } => {
                    format!("数据库错误: {}", source)
                }
                ExchangeEngineError::UnsupportedExchangeType {
                    exchange_type, account_id, ..
                } => {
                    format!("账户 {} 的交易所类型 {:?} 不支持", account_id, exchange_type)
                }
                ExchangeEngineError::Mt5Error { source, .. } => {
                    format!("MetaTrader5错误: {}", source.error_message(language))
                }
                ExchangeEngineError::ExchangeClientNotRegistered {
                    exchange_name, account_id, ..
                } => {
                    format!("客户端 {} 未注册。 客户端id: {}", exchange_name, account_id)
                }
                ExchangeEngineError::ExchangeClientOperationFailed {
                    message,
                    account_id,
                    operation,
                    exchange_type,
                    ..
                } => {
                    format!(
                        "账户 {} 交易所客户端操作失败: {}, 操作: {}, 交易所: {:?}",
                        account_id, message, operation, exchange_type
                    )
                }
                ExchangeEngineError::ExchangeClientTypeConversionFailed {
                    message,
                    account_id,
                    expected_type,
                    actual_type,
                    ..
                } => {
                    format!(
                        "账户 {} 交易所客户端类型转换失败: {}, 期望类型: {}, 实际类型: {}",
                        account_id, message, expected_type, actual_type
                    )
                }
                ExchangeEngineError::DatabaseOperationFailed {
                    message,
                    operation,
                    account_id,
                    table,
                    ..
                } => {
                    let mut msg = format!("数据库操作失败: {}, 操作: {}", message, operation);
                    if let Some(acc_id) = account_id {
                        msg.push_str(&format!(", 账户ID: {}", acc_id));
                    }
                    if let Some(t) = table {
                        msg.push_str(&format!(", 表: {}", t));
                    }
                    msg
                }
                ExchangeEngineError::DatabaseConnectionFailed { message, database_url, .. } => {
                    let url_str = if let Some(url) = database_url {
                        format!(", 数据库URL: {}", url)
                    } else {
                        String::new()
                    };
                    format!("数据库连接失败: {}{}", message, url_str)
                }
                ExchangeEngineError::OperationTimeout {
                    message,
                    account_id,
                    operation,
                    timeout_duration,
                    retry_count,
                    ..
                } => {
                    let mut msg = format!("操作超时: {}, 操作: {}, 超时时长: {}", message, operation, timeout_duration);
                    if let Some(acc_id) = account_id {
                        msg.push_str(&format!(", 账户: {}", acc_id));
                    }
                    if let Some(retry) = retry_count {
                        msg.push_str(&format!(", 重试次数: {}", retry));
                    }
                    msg
                }
                ExchangeEngineError::ConfigurationError {
                    message,
                    config_key,
                    account_id,
                    ..
                } => {
                    let mut msg = format!("配置错误: {}", message);
                    if let Some(key) = config_key {
                        msg.push_str(&format!(", 配置键: {}", key));
                    }
                    if let Some(acc_id) = account_id {
                        msg.push_str(&format!(", 账户ID: {}", acc_id));
                    }
                    msg
                }
                ExchangeEngineError::EnvironmentError {
                    message,
                    variable,
                    expected,
                    ..
                } => {
                    let mut msg = format!("环境错误: {}", message);
                    if let Some(var) = variable {
                        msg.push_str(&format!(", 变量: {}", var));
                    }
                    if let Some(exp) = expected {
                        msg.push_str(&format!(", 期望值: {}", exp));
                    }
                    msg
                }
                ExchangeEngineError::EventPublishingFailed {
                    message,
                    account_id,
                    event_type,
                    ..
                } => {
                    let mut msg = format!("事件发布失败: {}", message);
                    if let Some(acc_id) = account_id {
                        msg.push_str(&format!(", 账户ID: {}", acc_id));
                    }
                    if let Some(ev_type) = event_type {
                        msg.push_str(&format!(", 事件类型: {}", ev_type));
                    }
                    msg
                }
                ExchangeEngineError::CommandHandlingFailed {
                    message,
                    account_id,
                    command_type,
                    ..
                } => {
                    let mut msg = format!("命令处理失败: {}, 命令类型: {}", message, command_type);
                    if let Some(acc_id) = account_id {
                        msg.push_str(&format!(", 账户ID: {}", acc_id));
                    }
                    msg
                }
                ExchangeEngineError::ExchangeClientError { source } => {
                    format!("交易所客户端错误: {}", source.error_message(language))
                }
                ExchangeEngineError::Internal {
                    message,
                    component,
                    context,
                    account_id,
                    ..
                } => {
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
                }
                ExchangeEngineError::NotImplemented {
                    message,
                    feature,
                    exchange_type,
                    ..
                } => {
                    format!("功能未实现: {}, 功能: {}, 交易所类型: {:?}", message, feature, exchange_type)
                }
            },
        }
    }
}
