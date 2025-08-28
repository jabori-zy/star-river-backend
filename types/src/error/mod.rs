pub mod exchange_client_error;
pub mod error_trait;
pub mod engine;

use error_trait::StarRiverErrorTrait;

// Re-export commonly used error types
pub use engine::ExchangeEngineError;
pub use exchange_client_error::ExchangeClientError;

use thiserror::Error;


pub type ErrorCode = String;

/// Comprehensive top-level error type for the Star River Backend system
#[derive(Error, Debug)]
pub enum StarRiverError {
    // === Core Engine Errors ===
    /// Engine management errors
    #[error("ENGINE_MANAGER_ERROR: {message}")]
    EngineManager { 
        message: String,
        engine_name: Option<String>,
        operation: Option<String>,
    },
    
    /// Strategy engine errors
    #[error("STRATEGY_ENGINE_ERROR: {message}")]
    StrategyEngine {
        message: String,
        strategy_id: Option<i32>,
        strategy_name: Option<String>,
        node_id: Option<String>,
    },
    
    /// Market engine errors
    #[error("MARKET_ENGINE_ERROR: {message}")]
    MarketEngine {
        message: String,
        symbol: Option<String>,
        exchange: Option<String>,
    },
    
    /// Exchange engine errors  
    #[error("EXCHANGE_ENGINE_ERROR: {0}")]
    ExchangeEngine(#[from] ExchangeEngineError),
    
    /// Indicator engine errors
    #[error("INDICATOR_ENGINE_ERROR: {message}")]
    IndicatorEngine {
        message: String,
        indicator_name: Option<String>,
        symbol: Option<String>,
    },
    
    /// Cache engine errors
    #[error("CACHE_ENGINE_ERROR: {message}")]
    CacheEngine {
        message: String,
        cache_key: Option<String>,
        operation: Option<String>,
    },
    
    /// Account engine errors
    #[error("ACCOUNT_ENGINE_ERROR: {message}")]
    AccountEngine {
        message: String,
        account_id: Option<String>,
        operation: Option<String>,
    },

    // === Strategy System Errors ===
    /// Strategy node errors (general)
    #[error("STRATEGY_NODE_ERROR: {message}")]
    StrategyNode {
        message: String,
        node_type: Option<String>,
        node_id: Option<String>,
        strategy_id: Option<i32>,
    },
    
    /// Strategy validation errors
    #[error("STRATEGY_VALIDATION_ERROR: {message}")]
    StrategyValidation {
        message: String,
        strategy_id: Option<i32>,
        field: Option<String>,
    },
    
    /// Strategy execution errors
    #[error("STRATEGY_EXECUTION_ERROR: {message}")]
    StrategyExecution {
        message: String,
        strategy_id: Option<i32>,
        execution_phase: Option<String>,
    },

    // === Database Errors ===
    /// Database connection errors
    #[error("DATABASE_CONNECTION_ERROR: {message}")]
    DatabaseConnection {
        message: String,
        database_url: Option<String>,
    },
    
    /// Database operation errors
    #[error("DATABASE_OPERATION_ERROR: {message}")]
    DatabaseOperation {
        message: String,
        operation: Option<String>,
        table: Option<String>,
        entity_id: Option<String>,
    },
    
    /// Database migration errors
    #[error("DATABASE_MIGRATION_ERROR: {message}")]
    DatabaseMigration {
        message: String,
        migration_name: Option<String>,
    },
    
    /// SeaORM specific errors (when sea_orm is available)
    #[error("DATABASE_ORM_ERROR: {message}")]
    DatabaseORM {
        message: String,
        operation: Option<String>,
    },

    // === Exchange Integration Errors ===
    /// Binance exchange errors
    #[error("BINANCE_ERROR: {message}")]
    Binance {
        message: String,
        symbol: Option<String>,
        operation: Option<String>,
        error_code: Option<i32>,
    },
    
    /// MetaTrader5 errors
    #[error("METATRADER5_ERROR: {message}")]
    MetaTrader5 {
        message: String,
        terminal_id: Option<String>,
        symbol: Option<String>,
        operation: Option<String>,
        mt5_error_code: Option<i64>,
    },
    
    /// Generic exchange errors
    #[error("EXCHANGE_ERROR: {message}")]
    Exchange {
        message: String,
        exchange: Option<String>,
        operation: Option<String>,
    },

    // === Trading System Errors ===
    /// Order management errors
    #[error("ORDER_MANAGEMENT_ERROR: {message}")]
    OrderManagement {
        message: String,
        order_id: Option<String>,
        symbol: Option<String>,
        order_type: Option<String>,
    },
    
    /// Position management errors
    #[error("POSITION_MANAGEMENT_ERROR: {message}")]
    PositionManagement {
        message: String,
        position_id: Option<String>,
        symbol: Option<String>,
        operation: Option<String>,
    },
    
    /// Virtual trading system errors
    #[error("VIRTUAL_TRADING_ERROR: {message}")]
    VirtualTrading {
        message: String,
        operation: Option<String>,
        virtual_entity_id: Option<String>,
    },
    
    /// Account management errors
    #[error("ACCOUNT_MANAGEMENT_ERROR: {message}")]
    AccountManagement {
        message: String,
        account_id: Option<String>,
        operation: Option<String>,
    },

    // === Indicator System Errors ===  
    /// TA-Lib calculation errors
    #[error("TALIB_CALCULATION_ERROR: {message}")]
    TaLibCalculation {
        message: String,
        indicator_name: Option<String>,
        parameters: Option<String>,
        data_length: Option<usize>,
        lookback: Option<usize>,
    },
    
    /// Indicator data errors
    #[error("INDICATOR_DATA_ERROR: {message}")]
    IndicatorData {
        message: String,
        indicator_name: Option<String>,
        symbol: Option<String>,
        timeframe: Option<String>,
    },
    
    /// Indicator configuration errors
    #[error("INDICATOR_CONFIGURATION_ERROR: {message}")]
    IndicatorConfiguration {
        message: String,
        indicator_name: Option<String>,
        parameter: Option<String>,
    },

    // === Network & Communication Errors ===
    /// HTTP client errors
    #[error("HTTP_ERROR: {message}")]
    Http {
        message: String,
        url: Option<String>,
        status_code: Option<u16>,
        operation: Option<String>,
    },
    
    /// WebSocket errors
    #[error("WEBSOCKET_ERROR: {message}")]
    WebSocket {
        message: String,
        connection_id: Option<String>,
        operation: Option<String>,
    },
    
    /// Network connectivity errors
    #[error("NETWORK_ERROR: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
        operation: Option<String>,
    },
    
    /// Connection timeout errors
    #[error("TIMEOUT_ERROR: {message}")]
    Timeout {
        message: String,
        operation: Option<String>,
        timeout_duration: Option<String>,
    },

    // === Data Processing Errors ===
    /// JSON serialization/deserialization errors
    #[error("JSON_ERROR: {message}")]
    Json {
        message: String,
        field: Option<String>,
        data_type: Option<String>,
    },
    
    /// Data validation errors
    #[error("DATA_VALIDATION_ERROR: {message}")]
    DataValidation {
        message: String,
        field: Option<String>,
        value: Option<String>,
        expected: Option<String>,
    },
    
    /// Data conversion errors
    #[error("DATA_CONVERSION_ERROR: {message}")]
    DataConversion {
        message: String,
        from_type: Option<String>,
        to_type: Option<String>,
        value: Option<String>,
    },
    
    /// Market data processing errors
    #[error("MARKET_DATA_ERROR: {message}")]
    MarketData {
        message: String,
        symbol: Option<String>,
        data_type: Option<String>,
        timeframe: Option<String>,
    },

    // === Configuration Errors ===
    /// Application configuration errors
    #[error("CONFIGURATION_ERROR: {message}")]
    Configuration {
        message: String,
        config_key: Option<String>,
        config_file: Option<String>,
    },
    
    /// Environment setup errors
    #[error("ENVIRONMENT_ERROR: {message}")]
    Environment {
        message: String,
        variable: Option<String>,
        expected: Option<String>,
    },

    // === Authentication & Authorization ===
    /// Authentication errors
    #[error("AUTHENTICATION_ERROR: {message}")]
    Authentication {
        message: String,
        service: Option<String>,
        credential_type: Option<String>,
    },
    
    /// Authorization errors
    #[error("AUTHORIZATION_ERROR: {message}")]
    Authorization {
        message: String,
        resource: Option<String>,
        permission: Option<String>,
    },
    
    /// API rate limiting errors
    #[error("RATE_LIMIT_ERROR: {message}")]
    RateLimit {
        message: String,
        service: Option<String>,
        reset_time: Option<String>,
    },

    // === System & Resource Errors ===
    /// File system errors
    #[error("FILE_SYSTEM_ERROR: {message}")]
    FileSystem {
        message: String,
        path: Option<String>,
        operation: Option<String>,
    },
    
    /// Memory/resource errors
    #[error("RESOURCE_ERROR: {message}")]
    Resource {
        message: String,
        resource_type: Option<String>,
        operation: Option<String>,
    },
    
    /// Concurrency/threading errors
    #[error("CONCURRENCY_ERROR: {message}")]
    Concurrency {
        message: String,
        operation: Option<String>,
        thread_id: Option<String>,
    },

    // === Event System Errors ===
    /// Event publishing errors
    #[error("EVENT_PUBLISHING_ERROR: {message}")]
    EventPublishing {
        message: String,
        event_type: Option<String>,
        channel: Option<String>,
    },
    
    /// Event processing errors
    #[error("EVENT_PROCESSING_ERROR: {message}")]
    EventProcessing {
        message: String,
        event_type: Option<String>,
        handler: Option<String>,
    },
    
    /// Command handling errors
    #[error("COMMAND_HANDLING_ERROR: {message}")]
    CommandHandling {
        message: String,
        command_type: Option<String>,
        handler: Option<String>,
    },

    // === External Integration Errors ===
    /// Third-party service errors
    #[error("THIRD_PARTY_SERVICE_ERROR: {message}")]
    ThirdPartyService {
        message: String,
        service_name: Option<String>,
        operation: Option<String>,
        error_code: Option<String>,
    },

    // === Generic & Fallback Errors ===
    /// Feature not implemented
    #[error("FEATURE_NOT_IMPLEMENTED: {message}")]
    NotImplemented {
        message: String,
        feature: Option<String>,
    },
    
    /// Unsupported operation
    #[error("UNSUPPORTED_OPERATION: {message}")]
    UnsupportedOperation {
        message: String,
        operation: Option<String>,
        context: Option<String>,
    },
    
    /// Internal system errors
    #[error("INTERNAL_ERROR: {message}")]
    Internal {
        message: String,
        component: Option<String>,
        context: Option<String>,
    },
    
    /// Multiple errors occurred
    #[error("MULTIPLE_ERRORS: {}", .errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Multiple {
        errors: Vec<StarRiverError>,
        operation: Option<String>,
    },
}

impl StarRiverError {
    /// Returns the error prefix for Star River errors
    pub fn get_prefix(&self) -> &'static str {
        "STARRIVER"
    }
    
    /// Returns a string error code for Star River errors (format: STARRIVER_NNNN or nested error codes)
    pub fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            StarRiverError::ExchangeEngine(err) => err.error_code(),
            
            // For direct Star River errors, use STARRIVER prefix
            _ => {
                let prefix = self.get_prefix();
                let code = match self {
                    // Core Engine Errors (1001-1007)
                    StarRiverError::EngineManager { .. } => 1001,
                    StarRiverError::StrategyEngine { .. } => 1002,
                    StarRiverError::MarketEngine { .. } => 1003,
                    StarRiverError::IndicatorEngine { .. } => 1004,
                    StarRiverError::CacheEngine { .. } => 1005,
                    StarRiverError::AccountEngine { .. } => 1006,
                    
                    // Strategy System Errors (1007-1009)
                    StarRiverError::StrategyNode { .. } => 1007,
                    StarRiverError::StrategyValidation { .. } => 1008,
                    StarRiverError::StrategyExecution { .. } => 1009,
                    
                    // Database Errors (1010-1013)
                    StarRiverError::DatabaseConnection { .. } => 1010,
                    StarRiverError::DatabaseOperation { .. } => 1011,
                    StarRiverError::DatabaseMigration { .. } => 1012,
                    StarRiverError::DatabaseORM { .. } => 1013,
                    
                    // Exchange Integration Errors (1014-1016)
                    StarRiverError::Binance { .. } => 1014,
                    StarRiverError::MetaTrader5 { .. } => 1015,
                    StarRiverError::Exchange { .. } => 1016,
                    
                    // Trading System Errors (1017-1020)
                    StarRiverError::OrderManagement { .. } => 1017,
                    StarRiverError::PositionManagement { .. } => 1018,
                    StarRiverError::VirtualTrading { .. } => 1019,
                    StarRiverError::AccountManagement { .. } => 1020,
                    
                    // Indicator System Errors (1021-1023)
                    StarRiverError::TaLibCalculation { .. } => 1021,
                    StarRiverError::IndicatorData { .. } => 1022,
                    StarRiverError::IndicatorConfiguration { .. } => 1023,
                    
                    // Network & Communication Errors (1024-1027)
                    StarRiverError::Http { .. } => 1024,
                    StarRiverError::WebSocket { .. } => 1025,
                    StarRiverError::Network { .. } => 1026,
                    StarRiverError::Timeout { .. } => 1027,
                    
                    // Data Processing Errors (1028-1031)
                    StarRiverError::Json { .. } => 1028,
                    StarRiverError::DataValidation { .. } => 1029,
                    StarRiverError::DataConversion { .. } => 1030,
                    StarRiverError::MarketData { .. } => 1031,
                    
                    // System & Infrastructure Errors (1032-1043)
                    StarRiverError::Configuration { .. } => 1032,
                    StarRiverError::Environment { .. } => 1033,
                    StarRiverError::Authentication { .. } => 1034,
                    StarRiverError::Authorization { .. } => 1035,
                    StarRiverError::RateLimit { .. } => 1036,
                    StarRiverError::FileSystem { .. } => 1037,
                    StarRiverError::Resource { .. } => 1038,
                    StarRiverError::Concurrency { .. } => 1039,
                    StarRiverError::EventPublishing { .. } => 1040,
                    StarRiverError::EventProcessing { .. } => 1041,
                    StarRiverError::CommandHandling { .. } => 1042,
                    StarRiverError::ThirdPartyService { .. } => 1043,
                    
                    // Special Cases (1044-1047)
                    StarRiverError::NotImplemented { .. } => 1044,
                    StarRiverError::UnsupportedOperation { .. } => 1045,
                    StarRiverError::Internal { .. } => 1046,
                    StarRiverError::Multiple { .. } => 1047,
                    
                    // This should never happen due to outer match, but needed for completeness
                    StarRiverError::ExchangeEngine(_) => unreachable!(),
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }
    

    // === Engine Error Constructors ===
    pub fn engine_manager<S: Into<String>>(
        message: S, 
        engine_name: Option<String>, 
        operation: Option<String>
    ) -> Self {
        Self::EngineManager {
            message: message.into(),
            engine_name,
            operation,
        }
    }
    
    pub fn strategy_engine<S: Into<String>>(
        message: S,
        strategy_id: Option<i32>,
        strategy_name: Option<String>,
        node_id: Option<String>,
    ) -> Self {
        Self::StrategyEngine {
            message: message.into(),
            strategy_id,
            strategy_name,
            node_id,
        }
    }
    
    pub fn market_engine<S: Into<String>>(
        message: S,
        symbol: Option<String>,
        exchange: Option<String>,
    ) -> Self {
        Self::MarketEngine {
            message: message.into(),
            symbol,
            exchange,
        }
    }
    
    
    pub fn indicator_engine<S: Into<String>>(
        message: S,
        indicator_name: Option<String>,
        symbol: Option<String>,
    ) -> Self {
        Self::IndicatorEngine {
            message: message.into(),
            indicator_name,
            symbol,
        }
    }
    
    pub fn cache_engine<S: Into<String>>(
        message: S,
        cache_key: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::CacheEngine {
            message: message.into(),
            cache_key,
            operation,
        }
    }
    
    pub fn account_engine<S: Into<String>>(
        message: S,
        account_id: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::AccountEngine {
            message: message.into(),
            account_id,
            operation,
        }
    }

    // === Strategy System Error Constructors ===
    pub fn strategy_node<S: Into<String>>(
        message: S,
        node_type: Option<String>,
        node_id: Option<String>,
        strategy_id: Option<i32>,
    ) -> Self {
        Self::StrategyNode {
            message: message.into(),
            node_type,
            node_id,
            strategy_id,
        }
    }
    
    pub fn strategy_validation<S: Into<String>>(
        message: S,
        strategy_id: Option<i32>,
        field: Option<String>,
    ) -> Self {
        Self::StrategyValidation {
            message: message.into(),
            strategy_id,
            field,
        }
    }
    
    pub fn strategy_execution<S: Into<String>>(
        message: S,
        strategy_id: Option<i32>,
        execution_phase: Option<String>,
    ) -> Self {
        Self::StrategyExecution {
            message: message.into(),
            strategy_id,
            execution_phase,
        }
    }

    // === Database Error Constructors ===
    pub fn database_connection<S: Into<String>>(
        message: S,
        database_url: Option<String>,
    ) -> Self {
        Self::DatabaseConnection {
            message: message.into(),
            database_url,
        }
    }
    
    pub fn database_operation<S: Into<String>>(
        message: S,
        operation: Option<String>,
        table: Option<String>,
        entity_id: Option<String>,
    ) -> Self {
        Self::DatabaseOperation {
            message: message.into(),
            operation,
            table,
            entity_id,
        }
    }
    
    pub fn database_migration<S: Into<String>>(
        message: S,
        migration_name: Option<String>,
    ) -> Self {
        Self::DatabaseMigration {
            message: message.into(),
            migration_name,
        }
    }
    
    pub fn database_orm<S: Into<String>>(
        message: S,
        operation: Option<String>,
    ) -> Self {
        Self::DatabaseORM {
            message: message.into(),
            operation,
        }
    }

    // === Exchange Error Constructors ===
    pub fn binance<S: Into<String>>(
        message: S,
        symbol: Option<String>,
        operation: Option<String>,
        error_code: Option<i32>,
    ) -> Self {
        Self::Binance {
            message: message.into(),
            symbol,
            operation,
            error_code,
        }
    }
    
    pub fn metatrader5<S: Into<String>>(
        message: S,
        terminal_id: Option<String>,
        symbol: Option<String>,
        operation: Option<String>,
        mt5_error_code: Option<i64>,
    ) -> Self {
        Self::MetaTrader5 {
            message: message.into(),
            terminal_id,
            symbol,
            operation,
            mt5_error_code,
        }
    }
    
    pub fn exchange<S: Into<String>>(
        message: S,
        exchange: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::Exchange {
            message: message.into(),
            exchange,
            operation,
        }
    }

    // === Trading System Error Constructors ===
    pub fn order_management<S: Into<String>>(
        message: S,
        order_id: Option<String>,
        symbol: Option<String>,
        order_type: Option<String>,
    ) -> Self {
        Self::OrderManagement {
            message: message.into(),
            order_id,
            symbol,
            order_type,
        }
    }
    
    pub fn position_management<S: Into<String>>(
        message: S,
        position_id: Option<String>,
        symbol: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::PositionManagement {
            message: message.into(),
            position_id,
            symbol,
            operation,
        }
    }
    
    pub fn virtual_trading<S: Into<String>>(
        message: S,
        operation: Option<String>,
        virtual_entity_id: Option<String>,
    ) -> Self {
        Self::VirtualTrading {
            message: message.into(),
            operation,
            virtual_entity_id,
        }
    }
    
    pub fn account_management<S: Into<String>>(
        message: S,
        account_id: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::AccountManagement {
            message: message.into(),
            account_id,
            operation,
        }
    }

    // === Indicator Error Constructors ===
    pub fn talib_calculation<S: Into<String>>(
        message: S,
        indicator_name: Option<String>,
        parameters: Option<String>,
        data_length: Option<usize>,
        lookback: Option<usize>,
    ) -> Self {
        Self::TaLibCalculation {
            message: message.into(),
            indicator_name,
            parameters,
            data_length,
            lookback,
        }
    }
    
    pub fn indicator_data<S: Into<String>>(
        message: S,
        indicator_name: Option<String>,
        symbol: Option<String>,
        timeframe: Option<String>,
    ) -> Self {
        Self::IndicatorData {
            message: message.into(),
            indicator_name,
            symbol,
            timeframe,
        }
    }
    
    pub fn indicator_configuration<S: Into<String>>(
        message: S,
        indicator_name: Option<String>,
        parameter: Option<String>,
    ) -> Self {
        Self::IndicatorConfiguration {
            message: message.into(),
            indicator_name,
            parameter,
        }
    }

    // === Network Error Constructors ===
    pub fn http<S: Into<String>>(
        message: S,
        url: Option<String>,
        status_code: Option<u16>,
        operation: Option<String>,
    ) -> Self {
        Self::Http {
            message: message.into(),
            url,
            status_code,
            operation,
        }
    }
    
    pub fn websocket<S: Into<String>>(
        message: S,
        connection_id: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::WebSocket {
            message: message.into(),
            connection_id,
            operation,
        }
    }
    
    pub fn network<S: Into<String>>(
        message: S,
        endpoint: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            endpoint,
            operation,
        }
    }
    
    pub fn timeout<S: Into<String>>(
        message: S,
        operation: Option<String>,
        timeout_duration: Option<String>,
    ) -> Self {
        Self::Timeout {
            message: message.into(),
            operation,
            timeout_duration,
        }
    }

    // === Data Processing Error Constructors ===
    pub fn json<S: Into<String>>(
        message: S,
        field: Option<String>,
        data_type: Option<String>,
    ) -> Self {
        Self::Json {
            message: message.into(),
            field,
            data_type,
        }
    }
    
    pub fn data_validation<S: Into<String>>(
        message: S,
        field: Option<String>,
        value: Option<String>,
        expected: Option<String>,
    ) -> Self {
        Self::DataValidation {
            message: message.into(),
            field,
            value,
            expected,
        }
    }
    
    pub fn data_conversion<S: Into<String>>(
        message: S,
        from_type: Option<String>,
        to_type: Option<String>,
        value: Option<String>,
    ) -> Self {
        Self::DataConversion {
            message: message.into(),
            from_type,
            to_type,
            value,
        }
    }
    
    pub fn market_data<S: Into<String>>(
        message: S,
        symbol: Option<String>,
        data_type: Option<String>,
        timeframe: Option<String>,
    ) -> Self {
        Self::MarketData {
            message: message.into(),
            symbol,
            data_type,
            timeframe,
        }
    }

    // === Configuration Error Constructors ===
    pub fn configuration<S: Into<String>>(
        message: S,
        config_key: Option<String>,
        config_file: Option<String>,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            config_key,
            config_file,
        }
    }
    
    pub fn environment<S: Into<String>>(
        message: S,
        variable: Option<String>,
        expected: Option<String>,
    ) -> Self {
        Self::Environment {
            message: message.into(),
            variable,
            expected,
        }
    }

    // === Authentication Error Constructors ===
    pub fn authentication<S: Into<String>>(
        message: S,
        service: Option<String>,
        credential_type: Option<String>,
    ) -> Self {
        Self::Authentication {
            message: message.into(),
            service,
            credential_type,
        }
    }
    
    pub fn authorization<S: Into<String>>(
        message: S,
        resource: Option<String>,
        permission: Option<String>,
    ) -> Self {
        Self::Authorization {
            message: message.into(),
            resource,
            permission,
        }
    }
    
    pub fn rate_limit<S: Into<String>>(
        message: S,
        service: Option<String>,
        reset_time: Option<String>,
    ) -> Self {
        Self::RateLimit {
            message: message.into(),
            service,
            reset_time,
        }
    }

    // === System Error Constructors ===
    pub fn file_system<S: Into<String>>(
        message: S,
        path: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::FileSystem {
            message: message.into(),
            path,
            operation,
        }
    }
    
    pub fn resource<S: Into<String>>(
        message: S,
        resource_type: Option<String>,
        operation: Option<String>,
    ) -> Self {
        Self::Resource {
            message: message.into(),
            resource_type,
            operation,
        }
    }
    
    pub fn concurrency<S: Into<String>>(
        message: S,
        operation: Option<String>,
        thread_id: Option<String>,
    ) -> Self {
        Self::Concurrency {
            message: message.into(),
            operation,
            thread_id,
        }
    }

    // === Event System Error Constructors ===
    pub fn event_publishing<S: Into<String>>(
        message: S,
        event_type: Option<String>,
        channel: Option<String>,
    ) -> Self {
        Self::EventPublishing {
            message: message.into(),
            event_type,
            channel,
        }
    }
    
    pub fn event_processing<S: Into<String>>(
        message: S,
        event_type: Option<String>,
        handler: Option<String>,
    ) -> Self {
        Self::EventProcessing {
            message: message.into(),
            event_type,
            handler,
        }
    }
    
    pub fn command_handling<S: Into<String>>(
        message: S,
        command_type: Option<String>,
        handler: Option<String>,
    ) -> Self {
        Self::CommandHandling {
            message: message.into(),
            command_type,
            handler,
        }
    }

    // === External Integration Error Constructors ===
    pub fn third_party_service<S: Into<String>>(
        message: S,
        service_name: Option<String>,
        operation: Option<String>,
        error_code: Option<String>,
    ) -> Self {
        Self::ThirdPartyService {
            message: message.into(),
            service_name,
            operation,
            error_code,
        }
    }

    // === Generic Error Constructors ===
    pub fn not_implemented<S: Into<String>>(
        message: S,
        feature: Option<String>,
    ) -> Self {
        Self::NotImplemented {
            message: message.into(),
            feature,
        }
    }
    
    pub fn unsupported_operation<S: Into<String>>(
        message: S,
        operation: Option<String>,
        context: Option<String>,
    ) -> Self {
        Self::UnsupportedOperation {
            message: message.into(),
            operation,
            context,
        }
    }
    
    pub fn internal<S: Into<String>>(
        message: S,
        component: Option<String>,
        context: Option<String>,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            component,
            context,
        }
    }
    
    pub fn multiple(
        errors: Vec<StarRiverError>,
        operation: Option<String>,
    ) -> Self {
        Self::Multiple {
            errors,
            operation,
        }
    }
}

// === Context Trait for Enhanced Error Handling ===
/// Helper trait for adding context to errors
pub trait StarRiverErrorContext<T> {
    /// Add contextual information to an error
    fn with_context<F>(self, f: F) -> Result<T, StarRiverError>
    where
        F: FnOnce() -> String;
    
    /// Add component context to an error
    fn with_component_context(self, component: &str) -> Result<T, StarRiverError>;
    
    /// Add operation context to an error
    fn with_operation_context(self, operation: &str) -> Result<T, StarRiverError>;
    
    /// Add strategy context to an error
    fn with_strategy_context(self, strategy_id: i32, strategy_name: Option<String>) -> Result<T, StarRiverError>;
    
    /// Add symbol context to an error
    fn with_symbol_context(self, symbol: &str) -> Result<T, StarRiverError>;
}

impl<T, E> StarRiverErrorContext<T> for Result<T, E>
where
    E: Into<StarRiverError>,
{
    fn with_context<F>(self, f: F) -> Result<T, StarRiverError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            let context = f();
            StarRiverError::Internal {
                message: format!("{}: {}", context, base_error),
                component: None,
                context: Some(context),
            }
        })
    }
    
    fn with_component_context(self, component: &str) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            let base_error = e.into();
            StarRiverError::Internal {
                message: format!("Component '{}': {}", component, base_error),
                component: Some(component.to_string()),
                context: None,
            }
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            let base_error = e.into();
            StarRiverError::Internal {
                message: format!("Operation '{}': {}", operation, base_error),
                component: None,
                context: Some(operation.to_string()),
            }
        })
    }
    
    fn with_strategy_context(self, strategy_id: i32, strategy_name: Option<String>) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            let base_error = e.into();
            let strategy_info = match &strategy_name {
                Some(name) => format!("'{}'[{}]", name, strategy_id),
                None => strategy_id.to_string(),
            };
            StarRiverError::StrategyEngine {
                message: format!("Strategy {}: {}", strategy_info, base_error),
                strategy_id: Some(strategy_id),
                strategy_name,
                node_id: None,
            }
        })
    }
    
    fn with_symbol_context(self, symbol: &str) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            let base_error = e.into();
            StarRiverError::MarketData {
                message: format!("Symbol '{}': {}", symbol, base_error),
                symbol: Some(symbol.to_string()),
                data_type: None,
                timeframe: None,
            }
        })
    }
}

// === Utility Functions ===
impl StarRiverError {
    /// Check if the error is retriable (temporary failure)
    pub fn is_retriable(&self) -> bool {
        match self {
            StarRiverError::Network { .. } => true,
            StarRiverError::Timeout { .. } => true,
            StarRiverError::Http { status_code: Some(status), .. } if *status >= 500 => true,
            StarRiverError::Http { .. } => false,
            StarRiverError::RateLimit { .. } => true,
            StarRiverError::DatabaseConnection { .. } => true,
            StarRiverError::WebSocket { .. } => true,
            _ => false,
        }
    }
    
    /// Check if the error is a client error (4xx HTTP status codes or user input errors)
    pub fn is_client_error(&self) -> bool {
        match self {
            StarRiverError::Http { status_code: Some(status), .. } if *status >= 400 && *status < 500 => true,
            StarRiverError::DataValidation { .. } |
            StarRiverError::DataConversion { .. } |
            StarRiverError::StrategyValidation { .. } |
            StarRiverError::IndicatorConfiguration { .. } |
            StarRiverError::Configuration { .. } => true,
            _ => false,
        }
    }

    /// Extract structured context information from the error
    pub fn context(&self) -> Vec<(&'static str, String)> {
        match self {
            StarRiverError::EngineManager { engine_name, operation, .. } => {
                let mut ctx = vec![];
                if let Some(name) = engine_name {
                    ctx.push(("engine_name", name.clone()));
                }
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                ctx
            }
            StarRiverError::StrategyEngine { strategy_id, strategy_name, node_id, .. } => {
                let mut ctx = vec![];
                if let Some(id) = strategy_id {
                    ctx.push(("strategy_id", id.to_string()));
                }
                if let Some(name) = strategy_name {
                    ctx.push(("strategy_name", name.clone()));
                }
                if let Some(id) = node_id {
                    ctx.push(("node_id", id.clone()));
                }
                ctx
            }
            StarRiverError::MarketEngine { symbol, exchange, .. } => {
                let mut ctx = vec![];
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                if let Some(ex) = exchange {
                    ctx.push(("exchange", ex.clone()));
                }
                ctx
            }
            StarRiverError::ExchangeEngine(err) => err.context(),
            StarRiverError::IndicatorEngine { indicator_name, symbol, .. } => {
                let mut ctx = vec![];
                if let Some(name) = indicator_name {
                    ctx.push(("indicator_name", name.clone()));
                }
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                ctx
            }
            StarRiverError::DatabaseOperation { operation, table, entity_id, .. } => {
                let mut ctx = vec![];
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                if let Some(tbl) = table {
                    ctx.push(("table", tbl.clone()));
                }
                if let Some(id) = entity_id {
                    ctx.push(("entity_id", id.clone()));
                }
                ctx
            }
            StarRiverError::Binance { symbol, operation, error_code, .. } => {
                let mut ctx = vec![];
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                if let Some(code) = error_code {
                    ctx.push(("error_code", code.to_string()));
                }
                ctx
            }
            StarRiverError::MetaTrader5 { terminal_id, symbol, operation, mt5_error_code, .. } => {
                let mut ctx = vec![];
                if let Some(id) = terminal_id {
                    ctx.push(("terminal_id", id.clone()));
                }
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                if let Some(code) = mt5_error_code {
                    ctx.push(("mt5_error_code", code.to_string()));
                }
                ctx
            }
            StarRiverError::OrderManagement { order_id, symbol, order_type, .. } => {
                let mut ctx = vec![];
                if let Some(id) = order_id {
                    ctx.push(("order_id", id.clone()));
                }
                if let Some(sym) = symbol {
                    ctx.push(("symbol", sym.clone()));
                }
                if let Some(otype) = order_type {
                    ctx.push(("order_type", otype.clone()));
                }
                ctx
            }
            StarRiverError::Http { url, status_code, operation, .. } => {
                let mut ctx = vec![];
                if let Some(u) = url {
                    ctx.push(("url", u.clone()));
                }
                if let Some(code) = status_code {
                    ctx.push(("status_code", code.to_string()));
                }
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                ctx
            }
            StarRiverError::Multiple { errors, operation } => {
                let mut ctx = vec![("error_count", errors.len().to_string())];
                if let Some(op) = operation {
                    ctx.push(("operation", op.clone()));
                }
                ctx
            }
            // For other variants, return empty context or implement as needed
            _ => vec![],
        }
    }
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for StarRiverError {
    fn get_prefix(&self) -> &'static str {
        // For nested errors, delegate to the inner error's prefix
        match self {
            StarRiverError::ExchangeEngine(err) => err.get_prefix(),
            _ => self.get_prefix(),
        }
    }
    
    fn error_code(&self) -> ErrorCode {
        self.error_code()
    }
    
    fn context(&self) -> Vec<(&'static str, String)> {
        self.context()
    }
}

// Implement ErrorContext trait for StarRiverError
impl<T> crate::error::error_trait::ErrorContext<T, StarRiverError> for Result<T, StarRiverError> {
    fn with_context<F>(self, f: F) -> Result<T, StarRiverError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            StarRiverError::Internal {
                message: format!("{}: {}", context, e),
                component: None,
                context: Some(context),
            }
        })
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            StarRiverError::Internal {
                message: format!("Operation '{}': {}", operation, e),
                component: None,
                context: Some(format!("operation: {}", operation)),
            }
        })
    }
    
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, StarRiverError> {
        self.map_err(|e| {
            StarRiverError::Internal {
                message: format!("{} '{}': {}", resource_type, resource_id, e),
                component: None,
                context: Some(format!("{}: {}", resource_type, resource_id)),
            }
        })
    }
}

/// Result type alias for the Star River Backend system
pub type StarRiverResult<T> = Result<T, StarRiverError>;