# Star River Backend Error System Design

## Overview

The Star River Backend implements a comprehensive, hierarchical error handling system designed for a quantitative trading platform. The system provides structured error management with automatic conversions, rich contextual information, and domain-specific error handling patterns optimized for Rust's error handling idioms.

## Architecture

### Error Hierarchy

```
StarRiverError (Root) - Comprehensive top-level error
├── Core Engine Errors (1001-1006)
│   ├── EngineManager { engine_name, operation }
│   ├── StrategyEngine { strategy_id, strategy_name, node_id }
│   ├── MarketEngine { symbol, exchange }
│   ├── ExchangeEngine(ExchangeEngineError) - Nested delegation
│   ├── IndicatorEngine { indicator_name, symbol }
│   ├── CacheEngine { cache_key, operation }
│   └── AccountEngine { account_id, operation }
├── Strategy System Errors (1007-1009)
│   ├── StrategyNode { node_type, node_id, strategy_id }
│   ├── StrategyValidation { strategy_id, field }
│   └── StrategyExecution { strategy_id, execution_phase }
├── Database Errors (1010-1013)
│   ├── DatabaseConnection { database_url }
│   ├── DatabaseOperation { operation, table, entity_id }
│   ├── DatabaseMigration { migration_name }
│   └── DatabaseORM { operation }
├── Exchange Integration Errors (1014-1016)
│   ├── Binance { symbol, operation, error_code }
│   ├── MetaTrader5 { terminal_id, symbol, operation, mt5_error_code }
│   └── Exchange { exchange, operation }
├── Trading System Errors (1017-1020)
│   ├── OrderManagement { order_id, symbol, order_type }
│   ├── PositionManagement { position_id, symbol, operation }
│   ├── VirtualTrading { operation, virtual_entity_id }
│   └── AccountManagement { account_id, operation }
├── Indicator System Errors (1021-1023)
│   ├── TaLibCalculation { indicator_name, parameters, data_length, lookback }
│   ├── IndicatorData { indicator_name, symbol, timeframe }
│   └── IndicatorConfiguration { indicator_name, parameter }
├── Network & Communication Errors (1024-1027)
│   ├── Http { url, status_code, operation }
│   ├── WebSocket { connection_id, operation }
│   ├── Network { endpoint, operation }
│   └── Timeout { operation, timeout_duration }
├── Data Processing Errors (1028-1031)
│   ├── Json { field, data_type }
│   ├── DataValidation { field, value, expected }
│   ├── DataConversion { from_type, to_type, value }
│   └── MarketData { symbol, data_type, timeframe }
├── Configuration & Environment (1032-1033)
│   ├── Configuration { config_key, config_file }
│   └── Environment { variable, expected }
├── Authentication & Authorization (1034-1036)
│   ├── Authentication { service, credential_type }
│   ├── Authorization { resource, permission }
│   └── RateLimit { service, reset_time }
├── System & Resource Errors (1037-1039)
│   ├── FileSystem { path, operation }
│   ├── Resource { resource_type, operation }
│   └── Concurrency { operation, thread_id }
├── Event System Errors (1040-1042)
│   ├── EventPublishing { event_type, channel }
│   ├── EventProcessing { event_type, handler }
│   └── CommandHandling { command_type, handler }
├── External Integration (1043)
│   └── ThirdPartyService { service_name, operation, error_code }
└── Generic & Fallback Errors (1044-1047)
    ├── NotImplemented { feature }
    ├── UnsupportedOperation { operation, context }
    ├── Internal { component, context }
    └── Multiple { errors: Vec<StarRiverError>, operation }

ExchangeEngineError - Engine-level exchange operations
├── Registration & Configuration (1001-1005)
│   ├── RegisterExchangeFailed { account_id, exchange_type, source }
│   ├── UnregistrationFailed { account_id, exchange_type }
│   ├── Database { source: DbErr }
│   ├── UnsupportedExchangeType { exchange_type, account_id }
│   └── Mt5(Mt5Error) - Nested delegation
├── Exchange Client Management (1011-1013)
│   ├── ExchangeClientNotFound { account_id, requested_operation }
│   ├── ExchangeClientOperationFailed { account_id, operation, exchange_type }
│   └── ExchangeClientTypeConversionFailed { account_id, expected_type, actual_type }
├── Database Operations (1014-1015)
│   ├── DatabaseOperationFailed { operation, account_id, table }
│   └── DatabaseConnectionFailed { database_url }
├── Timeout & Configuration (1016-1018)
│   ├── OperationTimeout { account_id, operation, timeout_duration, retry_count }
│   ├── ConfigurationError { config_key, account_id }
│   └── EnvironmentError { variable, expected }
├── Event & Command Handling (1019-1020)
│   ├── EventPublishingFailed { account_id, event_type }
│   └── CommandHandlingFailed { account_id, command_type }
├── Nested Exchange Client (-)
│   └── ExchangeClientError(ExchangeClientError) - Nested delegation
└── Generic (1021-1022)
    ├── Internal { component, context, account_id }
    └── NotImplemented { feature, exchange_type }

ExchangeClientError - Client-level exchange operations
├── Exchange-Specific Errors (1001)
│   └── Binance(String)
├── Network & Connection (1003-1005)
│   ├── Network(String)
│   ├── WebSocket { exchange_type, account_id, url, source }
│   └── Timeout(String)
├── Authentication & Rate Limiting (1006-1007)
│   ├── Authentication(String)
│   └── RateLimit(String)
├── Data & Parameters (1008-1010)
│   ├── InvalidParameters(String)
│   ├── MarketData(String)
│   └── Serialization(String)
├── Trading Operations (1011-1013)
│   ├── OrderManagement(String)
│   ├── PositionManagement(String)
│   └── AccountManagement(String)
├── System Errors (1014-1017)
│   ├── Configuration(String)
│   ├── UnsupportedExchange(String)
│   ├── NotImplemented(String)
│   └── Internal(String)
└── Nested Errors (-)
    ├── MetaTrader5(Mt5Error) - Nested delegation
    └── DataProcessor(DataProcessorError) - Nested delegation

Mt5Error - MetaTrader5 specific errors
├── HTTP & Communication (1001-1006)
│   ├── Http { terminal_id, port, source: reqwest::Error }
│   ├── NoSuccessFieldInResponse
│   ├── HttpClientNotCreated { terminal_id, port }
│   ├── Json(serde_json::Error) - From conversion
│   ├── Connection { terminal_id, port }
│   └── Ping { terminal_id, port, source: Option<reqwest::Error> }
├── Market Data Operations (1007)
│   └── GetKlineData { symbol, code: Option<MT5ErrorCode> }
├── Trading Operations (1008-1012)
│   ├── CreateOrder { symbol, code: Option<MT5ErrorCode> }
│   ├── GetOrder { order_id }
│   ├── GetPosition { position_id }
│   ├── GetDeal { deal_id, position_id, order_id }
│   └── GetPositionNumber { symbol }
├── Account Operations (1013)
│   └── GetAccountInfo { terminal_id, port }
├── WebSocket & Connection (1014-1015)
│   ├── WebSocket { account_id, url, source }
│   └── Connection { terminal_id, port }
├── System Operations (1016-1022)
│   ├── InitializeTerminal(String)
│   ├── GetTerminalInfo(String)
│   ├── GetSymbolList(String)
│   ├── Initialization(String)
│   ├── Configuration(String)
│   ├── Server(String)
│   ├── Timeout(String)
│   ├── Authentication(String)
│   ├── Validation(String)
│   └── Internal(String)
└── Nested Data Processing (-)
    └── DataProcessor(DataProcessorError) - Nested delegation

DataProcessorError - Data processing and parsing errors
├── JSON & Stream Processing (1001-1002)
│   ├── JsonParsing(serde_json::Error) - From conversion
│   └── StreamProcessing { data_type }
├── Field & Data Structure (1003-1006)
│   ├── MissingField { field, context }
│   ├── InvalidFieldType { field, expected, actual, context }
│   ├── ArrayParsing { actual_type, context }
│   └── InvalidKlineArrayFormat { length, data }
├── Type Conversion & Validation (1007-1010)
│   ├── TypeConversion { field, value }
│   ├── DataValidation { field, value }
│   ├── EnumParsing { field, variant, valid_variants }
│   └── TimestampConversion { timestamp }
├── Specific Data Parsing (1011-1016)
│   ├── KlineDataParsing { symbol, interval }
│   ├── OrderDataParsing { order_id }
│   ├── PositionDataParsing { position_id }
│   ├── DealDataParsing { deal_id }
│   ├── AccountInfoParsing { account_id }
│   └── StreamDataFormat { expected_format, actual_data }
└── Internal Errors (1017)
    └── Internal(String)
```

### Core Traits

#### StarRiverErrorTrait
Provides unified error behavior across all error types:
```rust
/// Trait that all errors in the Star River Backend system should implement
/// This ensures consistent error handling patterns across all components
pub trait StarRiverErrorTrait: Error + Send + Sync + 'static {
    /// Returns the error prefix for this error type (e.g., "MT5", "DATA_PROCESSOR")
    fn get_prefix(&self) -> &'static str;
    
    /// Returns a string error code in format "PREFIX_NNNN" (e.g., "MT5_1001")
    fn error_code(&self) -> ErrorCode;
    
    /// Extract structured context information from the error
    /// Returns key-value pairs for logging, monitoring, and debugging
    fn context(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}
```

#### ErrorContext
Provides contextual error enhancement:
```rust
/// Helper trait for adding context to error results
pub trait ErrorContext<T, E>
where
    E: StarRiverErrorTrait,
{
    /// Add contextual information to an error result
    fn with_context<F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> String;
    
    /// Add operation context to an error result
    fn with_operation_context(self, operation: &str) -> Result<T, E>;
    
    /// Add resource context to an error result (symbol, order_id, etc.)
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, E>;
}
```

### Error Code Allocation

Error codes use string format with hierarchical prefixes to eliminate conflicts and provide clear error identification:

**String Format**: `PREFIX_NNNN` (e.g., `STARRIVER_1001`, `MT5_1007`, `DATA_PROCESSOR_1011`)

#### Current Error Code Ranges

- **StarRiverError**: `STARRIVER_NNNN` (1001-1047)
  - Core Engine Errors: 1001-1006
  - Strategy System Errors: 1007-1009
  - Database Errors: 1010-1013
  - Exchange Integration Errors: 1014-1016
  - Trading System Errors: 1017-1020
  - Indicator System Errors: 1021-1023
  - Network & Communication Errors: 1024-1027
  - Data Processing Errors: 1028-1031
  - Configuration & Environment: 1032-1033
  - Authentication & Authorization: 1034-1036
  - System & Resource Errors: 1037-1039
  - Event System Errors: 1040-1042
  - External Integration: 1043
  - Generic & Fallback Errors: 1044-1047
  - *Delegates to nested error codes when applicable*

- **ExchangeEngineError**: `EXCHANGE_ENGINE_NNNN` (1001-1022)
  - Registration & Configuration: 1001-1005
  - Exchange Client Management: 1011-1013
  - Database Operations: 1014-1015
  - Timeout & Configuration: 1016-1018
  - Event & Command Handling: 1019-1020
  - Generic: 1021-1022
  - *Delegates to nested error codes when applicable*

- **ExchangeClientError**: `EXCHANGE_CLIENT_NNNN` (1001-1017)
  - Exchange-Specific Errors: 1001
  - Network & Connection: 1003-1005
  - Authentication & Rate Limiting: 1006-1007
  - Data & Parameters: 1008-1010
  - Trading Operations: 1011-1013
  - System Errors: 1014-1017
  - *Delegates to nested error codes when applicable*

- **Mt5Error**: `MT5_NNNN` (1001-1022)
  - HTTP & Communication: 1001-1006
  - Market Data Operations: 1007
  - Trading Operations: 1008-1012
  - Account Operations: 1013
  - WebSocket & Connection: 1014-1015
  - System Operations: 1016-1022
  - *Delegates to nested error codes when applicable*

- **DataProcessorError**: `DATA_PROCESSOR_NNNN` (1001-1017)
  - JSON & Stream Processing: 1001-1002
  - Field & Data Structure: 1003-1006
  - Type Conversion & Validation: 1007-1010
  - Specific Data Parsing: 1011-1016
  - Internal Errors: 1017

#### Hierarchical Error Code Delegation

The error system uses **hierarchical delegation** where parent error types delegate to child error codes when nesting occurs. This preserves the most specific error information while maintaining a clear hierarchy.

**Example**: 
```rust
StarRiverError::ExchangeEngine(ExchangeEngineError::Mt5(Mt5Error::DataProcessor(DataProcessorError::JsonParsing(_))))
// Returns error code: "DATA_PROCESSOR_1001" (most specific)
```

**Key Benefits**:
- **No numeric collisions** possible due to unique prefixes
- **Hierarchical delegation** preserves most specific error information
- **Human-readable** and self-documenting format
- **API-friendly** structured error codes
- **Extensible** design allows adding new error types without conflicts

### Automatic Conversions

The system uses `#[from]` attributes and manual implementations for seamless error propagation:

#### From Attribute Conversions
```rust
#[derive(Error, Debug)]
pub enum StarRiverError {
    #[error("EXCHANGE_ENGINE_ERROR: {0}")]
    ExchangeEngine(#[from] ExchangeEngineError),
    // ... other variants
}

#[derive(Error, Debug)]
pub enum ExchangeEngineError {
    #[error("mt5 error: {0}")]
    Mt5(#[from] Mt5Error),
    
    #[error("exchange client error: {0}")]
    ExchangeClientError(#[from] ExchangeClientError),
    // ... other variants
}
```


#### Constructor Methods

Each error type provides ergonomic constructor methods:
```rust
impl StarRiverError {
    pub fn strategy_engine(
        message: impl Into<String>,
        strategy_id: Option<i32>,
        strategy_name: Option<String>,
        node_id: Option<String>,
    ) -> Self { /* ... */ }
    
    pub fn market_data(
        message: impl Into<String>,
        symbol: Option<String>,
        data_type: Option<String>,
        timeframe: Option<String>,
    ) -> Self { /* ... */ }
    
    // ... many more constructors for each error variant
}
```

## Current Implementation Status

### ✅ Completed Features
- **Comprehensive hierarchical error structure** with 47 distinct error types in StarRiverError
- **Automatic error conversions** with `#[from]` attributes throughout the hierarchy
- **Rich contextual error information** with structured context data
- **String-based hierarchical error codes** with unique prefixes (STARRIVER, MT5, DATA_PROCESSOR, etc.)
- **Trait-based unified error behavior** via StarRiverErrorTrait
- **Domain-specific error types** for MT5, data processing, exchange engines
- **Extensive From implementations** for standard Rust error types
- **Constructor methods** for ergonomic error creation
- **Context extraction** and **error enrichment** patterns
- **Retry logic support** with `is_retriable()` method
- **Client/server error classification** with `is_client_error()` method
- **Error aggregation** with Multiple variant supporting Vec<StarRiverError>

### 🔧 Areas for Improvement

#### 1. Memory and Performance Optimization
- **Current Issue**: Large enum variants with many optional fields may cause memory bloat
- **Current Issue**: Context methods allocate new vectors on every call
- **Suggested Solutions**:
  - Consider `Box<ErrorDetails>` for the largest variants if memory usage becomes problematic
  - Implement context caching for frequently accessed errors
  - Use `Cow<'static, str>` for common string literals

#### 2. Enhanced Error Aggregation
- **Current Status**: `StarRiverError::Multiple` exists but lacks dynamic management methods
- **Suggested Enhancement**: Add convenience methods for error aggregation
```rust
impl StarRiverError {
    pub fn add_error(self, other: StarRiverError) -> Self {
        match self {
            StarRiverError::Multiple { mut errors, operation } => {
                errors.push(other);
                StarRiverError::Multiple { errors, operation }
            }
            _ => StarRiverError::Multiple { 
                errors: vec![self, other], 
                operation: None 
            }
        }
    }
    
    pub fn aggregate(errors: Vec<StarRiverError>, operation: Option<String>) -> Self {
        StarRiverError::Multiple { errors, operation }
    }
}
```

#### 3. Serialization Support
- **Suggested Enhancement**: Add serde support for API error responses
```rust
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum StarRiverError {
    // ... variants
}
```

#### 4. Enhanced Standard Library Integration
- **Current Status**: Good coverage with From implementations for common types
- **Suggested Enhancement**: 
  - Add `From<Box<dyn std::error::Error>>` for dynamic error types
  - Consider integration patterns with `anyhow`/`eyre` for error chaining

## Recommendations

### High Priority
1. **Performance Profiling**: Measure actual memory usage in production scenarios
2. **Context Optimization**: Profile context() method performance under load

### Medium Priority  
1. **Add Serialization**: Implement serde support for API integration
2. **Enhanced Aggregation**: Implement error aggregation helper methods
3. **Documentation**: Add comprehensive module-level documentation

### Low Priority
1. **Integration**: Explore integration with popular error handling crates
2. **Tooling**: Consider build script for error code uniqueness validation

## Current Error Features

### Utility Methods
StarRiverError provides several utility methods for error classification:

```rust
impl StarRiverError {
    /// Check if the error is retriable (temporary failure)
    pub fn is_retriable(&self) -> bool {
        match self {
            StarRiverError::Network { .. } => true,
            StarRiverError::Timeout { .. } => true,
            StarRiverError::Http { status_code: Some(status), .. } if *status >= 500 => true,
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
        // Extensive pattern matching for context extraction
        // Returns structured key-value pairs for logging/monitoring
    }
}
```

## Usage Patterns

### Error Creation

#### Using Constructor Methods
```rust
// Using ergonomic constructor methods
let error = StarRiverError::strategy_engine(
    "Node validation failed", 
    Some(123), 
    Some("MyStrategy".to_string()),
    Some("start_node".to_string())
);

let error = StarRiverError::market_data(
    "Invalid kline data format",
    Some("EURUSD".to_string()),
    Some("kline".to_string()),
    Some("M1".to_string())
);
```

#### Automatic Conversion from Nested Errors
```rust
// Automatic conversion via #[from] attributes
let mt5_error = Mt5Error::get_kline_data("EURUSD", "Connection timeout", Some(1001));
let exchange_error: ExchangeEngineError = mt5_error.into(); // Automatic
let star_river_error: StarRiverError = exchange_error.into(); // Automatic

// Or directly
let result: Result<(), StarRiverError> = Err(mt5_error.into());
```

#### From Standard Rust Error Types
```rust
// Automatic conversion from standard types
let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
let star_river_error: StarRiverError = io_error.into(); // Becomes FileSystem error

let json_error = serde_json::from_str::<Value>("invalid json");
let star_river_error: StarRiverError = json_error.unwrap_err().into(); // Becomes Json error
```

### Error Context Enhancement

#### Adding Context During Error Propagation
```rust
use crate::error::error_trait::ErrorContext;

fn process_market_data(symbol: &str) -> Result<MarketData, StarRiverError> {
    // Operations that might fail, with context added at each step
    validate_symbol(symbol)
        .with_resource_context("symbol", symbol)?;
        
    fetch_kline_data(symbol)
        .with_operation_context("fetch_kline_data")
        .with_context(|| format!("Failed to fetch kline data for {}", symbol))?;
        
    Ok(market_data)
}
```

#### Using StarRiverErrorContext Helper
```rust
use crate::error::StarRiverErrorContext;

fn execute_strategy(strategy_id: i32, strategy_name: String) -> Result<(), StarRiverError> {
    let result = run_strategy_logic()
        .with_strategy_context(strategy_id, Some(strategy_name))
        .with_symbol_context("EURUSD")?;
        
    Ok(result)
}
```

### Error Handling and Analysis

#### Pattern Matching with Error Classification
```rust
fn handle_trading_error(error: StarRiverError) {
    match error {
        // Handle specific nested errors
        StarRiverError::ExchangeEngine(ExchangeEngineError::Mt5(mt5_err)) => {
            println!("MT5 Error: {}", mt5_err.error_code());
            if error.is_retriable() {
                schedule_retry();
            }
        }
        
        // Handle broad categories
        StarRiverError::OrderManagement { order_id, symbol, order_type, .. } => {
            log::error!("Order management failed: order_id={:?}, symbol={:?}, type={:?}", 
                       order_id, symbol, order_type);
        }
        
        // Use utility methods for classification
        error if error.is_client_error() => {
            respond_with_400_error(error);
        }
        
        error if error.is_retriable() => {
            schedule_retry_with_backoff(error);
        }
        
        _ => {
            log::error!("Unhandled error: {} [{}]", error, error.error_code());
        }
    }
}
```

#### Structured Context Extraction
```rust
fn log_error_details(error: &dyn StarRiverErrorTrait) {
    println!("Error Code: {}", error.error_code());
    
    // Extract structured context for monitoring/alerting
    let context = error.context();
    for (key, value) in context {
        println!("  {}: {}", key, value);
    }
    
    // Send to monitoring system
    send_to_monitoring_system(&error.error_code(), &context);
}
```

#### Multiple Error Handling
```rust
fn process_multiple_operations() -> Result<(), StarRiverError> {
    let mut errors = Vec::new();
    
    // Collect errors from multiple operations
    if let Err(e) = operation_1() {
        errors.push(e);
    }
    
    if let Err(e) = operation_2() {
        errors.push(e);
    }
    
    // Return aggregated error if any failed
    if !errors.is_empty() {
        return Err(StarRiverError::multiple(errors, Some("batch_operation".to_string())));
    }
    
    Ok(())
}
```

### API Integration Patterns

#### HTTP API Error Responses
```rust
fn api_error_response(error: StarRiverError) -> HttpResponse {
    let error_code = error.error_code();
    let context: std::collections::HashMap<String, String> = error.context().into_iter().collect();
    
    let response = ErrorResponse {
        code: error_code,
        message: error.to_string(),
        details: context,
        retriable: error.is_retriable(),
        timestamp: chrono::Utc::now(),
    };
    
    let status = if error.is_client_error() {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    
    HttpResponse::build(status).json(response)
}
```

## Context System: `context()` vs `with_context()`

The error system provides two distinct context mechanisms for different scenarios:

### **StarRiverErrorTrait::context()** - Error Introspection

**Purpose**: Extract structured context information **from existing errors**  
**When to use**: When you already have an error and want to analyze/log its details

```rust
/// Extract structured context information from the error
/// Returns key-value pairs for logging, monitoring, and debugging
fn context(&self) -> Vec<(&'static str, String)>
```

**Example Usage**:
```rust
// Error already exists - extract its context for logging/monitoring
let error = Mt5Error::get_kline_data("EURUSD", "Connection timeout", Some(1001));

let context = error.context(); 
// Returns: [("symbol", "EURUSD")]

// Use for structured logging
for (key, value) in context {
    log::info!("Error context: {}={}", key, value);
}
```

### **ErrorContext::with_context()** - Error Enrichment

**Purpose**: Add contextual information **when creating/propagating errors**  
**When to use**: When an operation fails and you want to add more context before returning the error

```rust
/// Add contextual information to an error result
fn with_context<F>(self, f: F) -> Result<T, E>
where
    F: FnOnce() -> String;
```

**Example Usage**:
```rust
// Operation fails - add context during error propagation
fn get_market_data(symbol: &str) -> Result<MarketData, StarRiverError> {
    http_client.fetch_data(symbol)
        .with_context(|| format!("Failed to fetch market data for {}", symbol))
        .with_operation_context("get_market_data") 
        .with_resource_context("symbol", symbol)
}
```

### **Key Differences**

| Aspect | `context()` | `with_context()` |
|--------|-------------|------------------|
| **Timing** | After error exists | During error creation/propagation |
| **Direction** | Extract context **from** error | Add context **to** error |
| **Use Case** | Logging, monitoring, debugging | Error enrichment, stack traces |
| **Return Type** | `Vec<(&str, String)>` | `Result<T, E>` |
| **Mutability** | Read-only introspection | Creates new enriched error |

### **Practical Scenarios**

#### **Error Logging/Monitoring** (Use `context()`)
```rust
fn log_error(error: &dyn StarRiverErrorTrait) {
    println!("Error: {} [{}]", error.error_code(), error);
    
    // Extract structured context for monitoring
    for (key, value) in error.context() {
        println!("  {}: {}", key, value);
    }
    
    // Send to monitoring system
    metrics::increment_counter("errors", &[
        ("code", &error.error_code()),
        ("prefix", error.get_prefix()),
    ]);
}
```

#### **Error Chain Building** (Use `with_context()`)
```rust
fn process_trading_order(order_id: i64) -> Result<(), StarRiverError> {
    // Each operation adds its own context
    validate_order(order_id)
        .with_resource_context("order", &order_id.to_string())?;
        
    execute_trade(order_id)
        .with_operation_context("execute_trade")
        .with_context(|| format!("Trade execution failed for order {}", order_id))?;
        
    update_portfolio(order_id)
        .with_operation_context("update_portfolio")?;
        
    Ok(())
}
```

#### **API Error Responses** (Use both)
```rust
fn handle_api_error(error: StarRiverError) -> HttpResponse {
    // Use context() to extract details for API response
    let details: HashMap<String, String> = error.context().into_iter().collect();
    
    HttpResponse::InternalServerError().json(ErrorResponse {
        code: error.error_code(),
        message: error.to_string(),
        details,
        retriable: error.is_retriable(),
    })
}

fn api_endpoint() -> Result<Json<Data>, StarRiverError> {
    database_operation()
        .with_context(|| "Database operation failed during API request")?; // Add context
        
    Ok(Json(data))
}
```

### **Summary**

- **`context()`**: **"What context does this error contain?"** - Read existing error details
- **`with_context()`**: **"Let me add context to this error"** - Enrich error with additional information

Both work together to provide a comprehensive error handling system where you can both **add context during error propagation** and **extract context for analysis** later.

## StarRiverResult Type Alias

The error system provides a convenient type alias for consistency across the codebase:

```rust
/// Result type alias for the Star River Backend system
pub type StarRiverResult<T> = Result<T, StarRiverError>;
```

This allows for cleaner function signatures throughout the system:

```rust
// Instead of
fn process_data() -> Result<Data, StarRiverError> { /* ... */ }

// You can write
fn process_data() -> StarRiverResult<Data> { /* ... */ }
```

## Design Philosophy

The Star River Backend error system embodies Rust's error handling philosophy while addressing the specific needs of a complex quantitative trading system:

### Core Principles

1. **Type Safety & Compile-Time Validation**: All errors are strongly typed with automatic conversion between error levels using `#[from]` attributes, ensuring no runtime surprises.

2. **Rich Contextual Information**: Every error carries structured metadata (symbol, account_id, operation, etc.) for effective debugging, monitoring, and alerting in production environments.

3. **Hierarchical Error Delegation**: Parent error types delegate to child error codes when nesting occurs, preserving the most specific error information while maintaining clear organizational hierarchy.

4. **Zero-Cost Abstractions**: Error propagation uses Rust's `?` operator with automatic conversions, minimizing boilerplate while maintaining performance.

5. **Domain-Driven Design**: Clear error boundaries separate concerns (MT5 client vs. data processing vs. engine management), making the codebase more maintainable.

6. **Operational Excellence**: Built-in support for retry logic (`is_retriable()`), client error classification (`is_client_error()`), and structured context extraction for monitoring systems.

7. **Developer Ergonomics**: Constructor methods, context enrichment traits, and comprehensive From implementations make error handling both powerful and pleasant to use.

8. **Financial System Reliability**: Error aggregation, detailed context tracking, and comprehensive coverage of trading operations (orders, positions, market data, etc.) ensure robust error handling in a high-stakes financial environment.

### Rust-Specific Error Handling Patterns

- **Error Trait Compliance**: All error types implement `std::error::Error` through `thiserror::Error`
- **Send + Sync**: All errors can be safely shared across threads
- **Automatic Display**: Human-readable error messages generated from structured data
- **Source Chain**: Nested error sources preserved for debugging and error analysis
- **Zero Allocation Context**: Context extraction designed to minimize allocations in hot paths

This design provides a robust, type-safe foundation for error handling in a complex financial trading system while leveraging Rust's zero-cost abstractions and maintaining excellent developer productivity.