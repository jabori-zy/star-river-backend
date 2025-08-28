use thiserror::Error;
use crate::error::ErrorCode;
use crate::error::exchange_client_error::ExchangeClientError;
use crate::custom_type::AccountId;
use crate::error::exchange_client_error::Mt5Error;
use crate::market::Exchange;
use sea_orm::error::DbErr;

#[derive(Error, Debug)]
pub enum ExchangeEngineError {
    // === Registration & Configuration Errors ===
    #[error("exchange registration failed for account {account_id}: {message}")]
    RegisterExchangeFailed {
        message: String,
        account_id: AccountId,
        exchange_type: Exchange,
        #[source]
        source: ExchangeClientError,
    },
    
    #[error("exchange unregistration failed for account {account_id}: {message}")]
    UnregistrationFailed {
        message: String,
        account_id: AccountId,
        exchange_type: Exchange,
    },
    

    #[error("database error: {message}")]
    Database { 
        message: String,
        #[source]
        source: DbErr,
    },
    
    #[error("unsupported exchange type: {exchange_type:?}, account_id: {account_id}")]
    UnsupportedExchangeType {
        message: String,
        exchange_type: Exchange,
        account_id: AccountId,
    },

    #[error("mt5 error: {0}")]
    Mt5(#[from] Mt5Error),
    
    // === Exchange Client Management Errors ===
    #[error("exchange client not found for account {account_id}")]
    ExchangeClientNotFound {
        message: String,
        account_id: AccountId,
        requested_operation: Option<String>,
    },
    
    #[error("exchange client operation failed for account {account_id}: {message}")]
    ExchangeClientOperationFailed {
        message: String,
        account_id: AccountId,
        operation: String,
        exchange_type: Exchange,
    },
    
    #[error("exchange client type conversion failed for account {account_id}: {message}")]
    ExchangeClientTypeConversionFailed {
        message: String,
        account_id: AccountId,
        expected_type: String,
        actual_type: String,
    },
    
    // === Database Errors ===
    #[error("database operation failed: {message}")]
    DatabaseOperationFailed {
        message: String,
        operation: String,
        account_id: Option<AccountId>,
        table: Option<String>,
    },
    
    #[error("database connection failed: {message}")]
    DatabaseConnectionFailed {
        message: String,
        database_url: Option<String>,
    },
    
    // === Timeout Errors ===
    #[error("operation timeout for account {}: {message}", account_id.map(|id| id.to_string()).unwrap_or("unknown".to_string()))]
    OperationTimeout {
        message: String,
        account_id: Option<AccountId>,
        operation: String,
        timeout_duration: String,
        retry_count: Option<u32>,
    },
    
    // === Configuration Errors ===
    #[error("configuration error: {message}")]
    ConfigurationError {
        message: String,
        config_key: Option<String>,
        account_id: Option<AccountId>,
    },
    
    #[error("environment error: {message}")]
    EnvironmentError {
        message: String,
        variable: Option<String>,
        expected: Option<String>,
    },
    
    // === Event & Command Handling Errors ===
    #[error("event publishing failed: {message}")]
    EventPublishingFailed {
        message: String,
        account_id: Option<AccountId>,
        event_type: Option<String>,
    },
    
    #[error("command handling failed: {message}")]
    CommandHandlingFailed {
        message: String,
        account_id: Option<AccountId>,
        command_type: String,
    },
    
    #[error("exchange client error: {0}")]
    ExchangeClientError(#[from] ExchangeClientError),
    
    // === Generic Errors ===
    #[error("internal exchange engine error: {message}")]
    Internal {
        message: String,
        component: Option<String>,
        context: Option<String>,
        account_id: Option<AccountId>,
    },
    
    #[error("feature not implemented: {message}")]
    NotImplemented {
        message: String,
        feature: String,
        exchange_type: Exchange,
    },
}

impl ExchangeEngineError {
    /// Returns the error prefix for exchange engine errors
    pub fn get_prefix(&self) -> &'static str {
        "EXCHANGE_ENGINE"
    }
    
    /// Returns a string error code for exchange engine errors (format: EXCHANGE_ENGINE_NNNN)
    pub fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeEngineError::ExchangeClientError(client_err) => client_err.error_code(),
            
            // For direct exchange engine errors, use EXCHANGE_ENGINE prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // Registration & Configuration (1001-1004)
                    ExchangeEngineError::RegisterExchangeFailed { .. } => 1001,
                    ExchangeEngineError::UnregistrationFailed { .. } => 1002,
                    ExchangeEngineError::Database { .. } => 1003,
                    ExchangeEngineError::UnsupportedExchangeType { .. } => 1004,

                    // Mt5 (1005-1009)
                    ExchangeEngineError::Mt5(_) => 1005,
                    
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
                    ExchangeEngineError::ExchangeClientError(_) => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }
}

// Implement the StarRiverErrorTrait for ExchangeEngineError
impl crate::error::error_trait::StarRiverErrorTrait for ExchangeEngineError {
    fn get_prefix(&self) -> &'static str {
        // For nested errors, delegate to the inner error's prefix
        match self {
            ExchangeEngineError::ExchangeClientError(client_err) => client_err.get_prefix(),
            _ => self.get_prefix(),
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn context(&self) -> Vec<(&'static str, String)> {
        match self {
            ExchangeEngineError::RegisterExchangeFailed { account_id, exchange_type, .. } => {
                let mut ctx = vec![("account_id", account_id.to_string())];
                ctx.push(("exchange_type", exchange_type.to_string()));
                ctx
            },
            
            ExchangeEngineError::ExchangeClientOperationFailed { account_id, operation, exchange_type, .. } => {
                let mut ctx = vec![
                    ("account_id", account_id.to_string()),
                    ("operation", operation.clone())
                ];
                ctx.push(("exchange_type", exchange_type.to_string()));
                ctx
            },
            ExchangeEngineError::DatabaseOperationFailed { operation, account_id, table, .. } => {
                let mut ctx = vec![("operation", operation.clone())];
                if let Some(acc_id) = account_id {
                    ctx.push(("account_id", acc_id.to_string()));
                }
                if let Some(t) = table {
                    ctx.push(("table", t.clone()));
                }
                ctx
            },
            ExchangeEngineError::OperationTimeout { account_id, operation, timeout_duration, retry_count, .. } => {
                let mut ctx = vec![
                    ("operation", operation.clone()),
                    ("timeout_duration", timeout_duration.clone())
                ];
                if let Some(acc_id) = account_id {
                    ctx.push(("account_id", acc_id.to_string()));
                }
                if let Some(retry) = retry_count {
                    ctx.push(("retry_count", retry.to_string()));
                }
                ctx
            },
            ExchangeEngineError::EventPublishingFailed { account_id, event_type, .. } => {
                let mut ctx = vec![];
                if let Some(acc_id) = account_id {
                    ctx.push(("account_id", acc_id.to_string()));
                }
                if let Some(ev_type) = event_type {
                    ctx.push(("event_type", ev_type.clone()));
                }
                ctx
            },
            ExchangeEngineError::CommandHandlingFailed { account_id, command_type, .. } => {
                let mut ctx = vec![("command_type", command_type.clone())];
                if let Some(acc_id) = account_id {
                    ctx.push(("account_id", acc_id.to_string()));
                }
                ctx
            },
            _ => {
                // For simpler error types, extract account_id if available
                match self {
                    ExchangeEngineError::UnregistrationFailed { account_id, .. } |
                    ExchangeEngineError::ExchangeClientNotFound { account_id, .. } => {
                        vec![("account_id", account_id.to_string())]
                    },
                    ExchangeEngineError::ExchangeClientError(_) => {
                        vec![]
                    },
                    _ => vec![],
                }
            }
        }
    }
}

// Implement ErrorContext trait for ExchangeEngineError
impl<T> crate::error::error_trait::ErrorContext<T, ExchangeEngineError> for Result<T, ExchangeEngineError> {
    fn with_context<F>(self, f: F) -> Result<T, ExchangeEngineError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            ExchangeEngineError::Internal {
                message: format!("{}: {}", context, e),
                component: None,
                context: Some(context),
                account_id: None,
            }
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, ExchangeEngineError> {
        self.map_err(|e| {
            ExchangeEngineError::Internal {
                message: format!("Exchange Engine Operation '{}': {}", operation, e),
                component: Some("exchange_engine".to_string()),
                context: Some(operation.to_string()),
                account_id: None,
            }
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, ExchangeEngineError> {
        self.map_err(|e| {
            ExchangeEngineError::Internal {
                message: format!("Exchange Engine {} '{}': {}", resource_type, resource_id, e),
                component: Some("exchange_engine".to_string()),
                context: Some(format!("{}:{}", resource_type, resource_id)),
                account_id: None,
            }
        })
    }
}

impl ExchangeEngineError {
    // === Registration Error Constructors ===
    pub fn register_exchange_failed<S: Into<String>>(
        message: S, 
        account_id: AccountId,
        exchange_type: Exchange,
        source: ExchangeClientError
    ) -> Self {
        Self::RegisterExchangeFailed {
            message: message.into(),
            account_id,
            exchange_type,
            source,
        }
    }
    
    pub fn unregistration_failed<S: Into<String>>(
        message: S,
        account_id: AccountId,
        exchange_type: Exchange
    ) -> Self {
        Self::UnregistrationFailed {
            message: message.into(),
            account_id,
            exchange_type,
        }
    }
    

    
    pub fn unsupported_exchange_type<S: Into<String>>(
        message: S,
        exchange_type: Exchange,
        account_id: AccountId
    ) -> Self {
        Self::UnsupportedExchangeType {
            message: message.into(),
            exchange_type,
            account_id,
        }
    }

    
    // === Exchange Client Error Constructors ===
    pub fn exchange_client_not_found<S: Into<String>>(
        message: S,
        account_id: AccountId,
        requested_operation: Option<String>
    ) -> Self {
        Self::ExchangeClientNotFound {
            message: message.into(),
            account_id,
            requested_operation,
        }
    }
    
    pub fn exchange_client_operation_failed<S: Into<String>>(
        message: S,
        account_id: AccountId,
        operation: String,
        exchange_type: Exchange
    ) -> Self {
        Self::ExchangeClientOperationFailed {
            message: message.into(),
            account_id,
            operation,
            exchange_type,
        }
    }
    
    // === Database Error Constructors ===
    pub fn database_operation_failed<S: Into<String>>(
        message: S,
        operation: String,
        account_id: Option<AccountId>,
        table: Option<String>
    ) -> Self {
        Self::DatabaseOperationFailed {
            message: message.into(),
            operation,
            account_id,
            table,
        }
    }
    
    pub fn database_connection_failed<S: Into<String>>(
        message: S,
        database_url: Option<String>
    ) -> Self {
        Self::DatabaseConnectionFailed {
            message: message.into(),
            database_url,
        }
    }
    
    // === Timeout Error Constructors ===
    pub fn operation_timeout<S: Into<String>>(
        message: S,
        account_id: Option<AccountId>,
        operation: String,
        timeout_duration: String,
        retry_count: Option<u32>
    ) -> Self {
        Self::OperationTimeout {
            message: message.into(),
            account_id,
            operation,
            timeout_duration,
            retry_count,
        }
    }
    
    // === Configuration Error Constructors ===
    pub fn configuration_error<S: Into<String>>(
        message: S,
        config_key: Option<String>,
        account_id: Option<AccountId>
    ) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            config_key,
            account_id,
        }
    }
    
    // === Generic Error Constructors ===
    pub fn internal<S: Into<String>>(
        message: S,
        component: Option<String>,
        context: Option<String>,
        account_id: Option<AccountId>
    ) -> Self {
        Self::Internal {
            message: message.into(),
            component,
            context,
            account_id,
        }
    }
    
    pub fn not_implemented<S: Into<String>>(
        message: S,
        feature: String,
        exchange_type: Exchange
    ) -> Self {
        Self::NotImplemented {
            message: message.into(),
            feature,
            exchange_type,
        }
    }
    
    // === Event & Command Error Constructors ===
    pub fn event_publishing_failed<S: Into<String>>(
        message: S,
        account_id: Option<AccountId>,
        event_type: Option<String>
    ) -> Self {
        Self::EventPublishingFailed {
            message: message.into(),
            account_id,
            event_type,
        }
    }
    
    pub fn command_handling_failed<S: Into<String>>(
        message: S,
        account_id: Option<AccountId>,
        command_type: String
    ) -> Self {
        Self::CommandHandlingFailed {
            message: message.into(),
            account_id,
            command_type,
        }
    }
    
}