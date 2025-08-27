# Star River Backend Error System Design

## Overview

The Star River Backend implements a comprehensive, hierarchical error handling system designed for a quantitative trading platform. The system provides structured error management with automatic conversions, rich contextual information, and compile-time validation.

## Architecture

### Error Hierarchy

```
StarRiverError (Root)
├── EngineError
│   ├── ExchangeEngine(ExchangeEngineError)
│   ├── Strategy(StrategyEngineError) 
│   ├── Market(MarketEngineError)
│   ├── Cache(CacheEngineError)
│   ├── Account(AccountEngineError)
│   └── Indicator(IndicatorEngineError)
├── ExchangeClient(ExchangeClientError)
│   ├── MetaTrader5(Mt5Error)
│   │   ├── HttpClient(Mt5HttpClientError)
│   │   └── DataProcessor(DataProcessorError)
│   └── Binance(BinanceError)
├── Database(DatabaseError)
├── Configuration(ConfigError)
├── Internal(String)
└── Multiple { errors: Vec<StarRiverError>, operation: Option<String> }
```

### Core Traits

#### StarRiverErrorTrait
Provides unified error behavior across all error types:
```rust
pub trait StarRiverErrorTrait {
    fn error_code(&self) -> ErrorCode;           // Unique numeric identifier
    fn category(&self) -> &'static str;          // Error category for classification
    fn is_retriable(&self) -> bool;              // Retry logic support
    fn is_client_error(&self) -> bool;           // Client vs server error distinction
    fn message(&self) -> &str;                   // Human-readable message
    fn context(&self) -> Vec<(&'static str, String)>; // Structured context data
}
```

#### ErrorContext
Provides contextual error enhancement:
```rust
pub trait ErrorContext<T, E> {
    fn with_context<F>(self, f: F) -> Result<T, E> where F: FnOnce() -> String;
    fn with_operation_context(self, operation: &str) -> Result<T, E>;
    fn with_resource_context(self, resource_type: &str, resource_id: &str) -> Result<T, E>;
}
```

### Error Code Allocation

Error codes now use string format with prefixes to eliminate conflicts:

**String Format**: `PREFIX_NNNN` (e.g., `MT5_HTTP_CLIENT_1001`, `DATA_PROCESSOR_1005`)

- **StarRiverError**: `STARRIVER_NNNN` (1001-1047)
  - Delegates to nested error codes when applicable
- **ExchangeEngineError**: `EXCHANGE_ENGINE_NNNN` (1001-1022)  
  - Delegates to nested error codes when applicable
- **ExchangeClientError**: `EXCHANGE_CLIENT_NNNN` (1001-1017)
  - Delegates to nested error codes when applicable
- **Mt5Error**: `MT5_NNNN` (1001-1010)
  - Delegates to nested error codes when applicable
- **DataProcessorError**: `DATA_PROCESSOR_NNNN` (1001-1017)
- **Mt5HttpClientError**: `MT5_HTTP_CLIENT_NNNN` (1001-1014)

**Key Benefits**:
- No numeric collisions possible due to unique prefixes
- Hierarchical delegation preserves most specific error information
- Human-readable and self-documenting
- API-friendly format

### Automatic Conversions

The system uses `#[from]` attributes for seamless error propagation:
```rust
#[derive(Error, Debug)]
pub enum StarRiverError {
    #[error("Exchange engine error: {0}")]
    ExchangeEngine(#[from] ExchangeEngineError),
    // ... other variants
}
```

### Build-time Validation

A comprehensive build script (`build.rs`) validates error code uniqueness across the entire system, preventing duplicate codes at compile time.

## Current Implementation Status

### ✅ Completed Features
- Hierarchical error structure
- Automatic error conversions with `#[from]`
- Rich contextual error information
- Comprehensive error code allocation
- Build-time duplicate detection
- Trait-based error behavior
- Domain-specific error types (MT5, data processing, engines)
- **String-based hierarchical error codes with prefixes** ✨ *Recently Implemented*
- **StarRiverErrorTrait implementation for StarRiverError** ✨ *Recently Fixed*
- **Fixed ErrorContext implementation in Mt5HttpClientError** ✨ *Recently Fixed*

### ❌ Critical Issues Identified



### 🔧 Medium Priority Issues

#### 1. Memory and Performance Issues
- Large enum variants cause memory bloat
- Context methods allocate vectors on every call
- Excessive string cloning

**Solutions**:
- Use `Box<ErrorDetails>` for complex variants
- Cache context information
- Use `Cow<'static, str>` for common strings

#### 2. Missing Error Aggregation
**Problem**: `StarRiverError::Multiple` lacks dynamic error management
**Solution**: Add error aggregation methods
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
}
```

#### 3. Poor Message Implementation
**Problem**: Generic messages instead of detailed error information
**Solution**: Return actual error details in `message()` methods
```rust
// Instead of generic "Required field missing"
DataProcessorError::MissingField { field, .. } => {
    format!("Required field '{}' missing", field)
}
```

### 🚀 Enhancement Opportunities

#### 4. Missing Serialization Support
**Add**: `#[derive(Serialize)]` for API error responses

#### 5. Limited Standard Library Integration
**Add**: 
- `From<Box<dyn std::error::Error>>` implementations
- Integration with `anyhow`/`eyre` for error chaining

#### 6. Documentation Gaps
**Add**:
- Module-level documentation
- Usage examples
- Error code range documentation

## Recommendations

### Immediate Actions (HIGH Priority)
1. **Update build script** for string-based error code validation

### Medium-term Improvements
1. **Add serialization support** for API integration
2. **Implement proper error aggregation**
3. **Optimize memory usage** and performance
4. **Enhance message extraction** methods

### Long-term Enhancements
1. **Comprehensive documentation** and examples
2. **Standardize naming conventions**
3. **Integration with popular error handling crates**

## Build Script Integration

The `build.rs` script provides compile-time validation:
- Parses all error files for error code extraction
- Detects duplicates across the entire system
- Fails compilation on conflicts
- Provides detailed conflict location information

## Usage Patterns

### Basic Error Creation
```rust
// Automatic conversion
let result: Result<(), StarRiverError> = Err(ExchangeEngineError::initialization("Failed to start"));

// With context
let result = some_operation()
    .with_operation_context("initialize_exchange")
    .with_resource_context("exchange", "mt5");
```

### Error Handling
```rust
match error {
    StarRiverError::ExchangeClient(ExchangeClientError::MetaTrader5(mt5_err)) => {
        if mt5_err.is_retriable() {
            retry_operation();
        }
    }
    _ => handle_other_errors(),
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
let error = Mt5HttpClientError::GetKlineData { 
    symbol: "EURUSD".to_string(), 
    message: "Connection timeout".to_string(), 
    code: Some(1001) 
};

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
    println!("Error: {} [{}]", error.error_code(), error.message());
    
    // Extract structured context for monitoring
    for (key, value) in error.context() {
        println!("  {}: {}", key, value);
    }
    
    // Send to monitoring system
    metrics::increment_counter("errors", &[
        ("code", &error.error_code()),
        ("category", error.category()),
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
        message: error.message().to_string(),
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

## Design Philosophy

The error system follows these principles:
1. **Type Safety**: Compile-time error checking and validation
2. **Rich Context**: Detailed error information for debugging
3. **Automatic Propagation**: Minimal boilerplate with `#[from]` attributes
4. **Domain Separation**: Clear error boundaries between system components
5. **Operational Awareness**: Support for retry logic and error classification
6. **Performance**: Efficient error handling without sacrificing detail

This design provides a robust foundation for error handling in a complex financial trading system while maintaining developer productivity and system reliability.