# Snafu Error Handling Q&A

This document summarizes questions and answers about using snafu for error handling in the Star River Backend project.

## 1. Q: How about using snafu? It natively supports context().

**A:** Snafu is an excellent choice for solving the error context problem. It offers several key advantages:

### Main Benefits:
- **Native context support** - No loss of error variants when adding context
- **Semantic error backtrace** - Each error provides complete context chain  
- **Type safety** - Compile-time error context validation
- **Preserves error information** - Unlike custom `ErrorContext` trait that converts everything to `Internal` errors

### Implementation approach:
```rust
use snafu::{Snafu, ResultExt};

#[derive(Debug, Snafu)]
pub enum Mt5Error {
    #[snafu(display("Failed to initialize terminal: {message}"))]
    InitializeTerminal {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
        backtrace: snafu::Backtrace,
    },
}

// Usage with context preservation
mt5.initialize_terminal().await
    .context(InitializeTerminalSnafu { message: "startup failed" })?;
```

This preserves the original error type while adding meaningful context.

---

## 2. Q: After using snafu, is it necessary to keep the context() method?

**A:** Yes, keep the `context()` method in `StarRiverErrorTrait` but change its purpose.

### Two Different "Context" Concepts:
1. **Snafu's `.context()`** - Adds contextual error wrapping while preserving error chains
2. **StarRiverErrorTrait's `context()`** - Extracts structured metadata for logging/monitoring

### Why keep it:
- **Structured logging** - Returns key-value pairs for telemetry systems
- **Monitoring integration** - Provides consistent metadata extraction
- **Debugging data** - Extracts specific fields like `symbol`, `order_id`, etc.

### Example of both working together:
```rust
// Snafu context for error chaining
mt5.get_kline_data("EURUSD").await
    .context(MarketDataOperationSnafu { operation: "fetch klines" })?;

// StarRiverErrorTrait context for structured data
let error_metadata = error.context(); // Returns HashMap with structured data
tracing::error!(
    error_code = %error.error_code(),
    metadata = ?error_metadata,
    "Market data operation failed"
);
```

**Recommendation:** Keep the method but change return type to `HashMap<&'static str, String>` for better usability.

---

## 3. Q: Change the context() method return type to HashMap

**A:** Changed from `Vec<(&'static str, String)>` to `HashMap<&'static str, String>` for better API:

### Before:
```rust
fn context(&self) -> Vec<(&'static str, String)> {
    vec![("symbol", symbol.clone())]
}
```

### After:
```rust
fn context(&self) -> HashMap<&'static str, String> {
    let mut ctx = HashMap::new();
    ctx.insert("symbol", symbol.clone());
    ctx
}
```

### Benefits:
- **Direct key access** - `error.context().get("symbol")` instead of searching Vec
- **Better performance** - O(1) lookup vs O(n) search
- **More intuitive** - Natural key-value semantics
- **Integration friendly** - Works better with logging frameworks

---

## 4. Q: What is the function of #[snafu(source(false))]?

**A:** `#[snafu(source(false))]` tells snafu to **not treat this field as an error source** in the error chain.

### Problem it solves:
```rust
#[derive(Debug, Snafu)]
pub enum Mt5Error {
    Ping {
        message: String,
        terminal_id: i32,
        port: u16,
        #[snafu(source(false))]  // Required here!
        source: Option<reqwest::Error>, // Option<Error> doesn't implement Error trait
        backtrace: Backtrace,
    },
}
```

### Why needed:
1. `Option<reqwest::Error>` doesn't implement `std::error::Error`
2. Snafu was trying to treat it as an error source and failing
3. `source(false)` tells snafu: "This is just data, not an error source"

### Alternative approaches:
```rust
// Option 1: Keep optional as data (current)
#[snafu(source(false))]
source: Option<reqwest::Error>,

// Option 2: Make it always required
source: reqwest::Error,

// Option 3: Separate variants
PingWithError { source: reqwest::Error, ... },
PingWithoutError { ... },
```

---

## 5. Q: What is the function of unstable-provider-api?

**A:** The `unstable-provider-api` feature enables `#[snafu(provide)]` annotation for Rust's experimental error provider API (RFC 3192).

### What it does:
```rust
#[derive(Debug, Snafu)]
pub enum MyError {
    DatabaseError {
        #[snafu(provide)]
        query_id: u64,        // Can be provided as u64
        
        #[snafu(provide)]
        table_name: String,   // Can be provided as String
        
        source: sqlx::Error,
    }
}
```

### Usage scenarios:
```rust
use std::error::request_ref;

fn handle_error(err: &dyn std::error::Error) {
    if let Some(query_id) = request_ref::<u64>(err) {
        println!("Failed query ID: {}", query_id);
    }
}
```

### Why not widely used:
- **Unstable API** - Requires nightly Rust
- **Limited ecosystem** - Few libraries support it
- **Alternative approaches** - Custom traits like `StarRiverErrorTrait` work better

**Recommendation:** Stick with current `context()` approach for stable Rust compatibility.

---

## 6. Q: How to use the context selector in snafu?

**A:** Context selectors are automatically generated structs that help add context to errors.

### How they work:
```rust
#[derive(Debug, Snafu)]
pub enum Mt5Error {
    #[snafu(display("Failed to get kline data for symbol '{symbol}': {message}"))]
    GetKlineData { 
        symbol: String, 
        message: String, 
        code: Option<MT5ErrorCode>,
        backtrace: Backtrace,
    },
}

// Snafu automatically generates:
pub struct GetKlineDataSnafu {
    pub symbol: String,
    pub message: String,
    pub code: Option<MT5ErrorCode>,
    // Note: No backtrace/source - handled automatically
}
```

### Usage:
```rust
use snafu::ResultExt;

async fn fetch_kline_data(symbol: &str) -> Result<KlineData, Mt5Error> {
    let data = http_client.get_klines(symbol).await
        .context(GetKlineDataSnafu {
            symbol: symbol.to_string(),
            message: "Failed to fetch from MT5 API".to_string(),
            code: None,
        })?;
    
    Ok(data)
}
```

### Key benefits:
- **Automatic error chaining** - Original error becomes source
- **Automatic backtrace** - Generated automatically
- **Type safety** - Can't forget required fields
- **Clean syntax** - Less boilerplate than manual construction

---

## 7. Q: What's the difference between GetKlineData directly vs GetKlineDataSnafu?

**A:** This is a crucial distinction:

### GetKlineData - The Error Variant:
```rust
Mt5Error::GetKlineData {
    symbol: String,
    message: String,
    code: Option<MT5ErrorCode>,
    source: reqwest::Error,      // Must set manually
    backtrace: Backtrace,        // Must generate manually
}
```

### GetKlineDataSnafu - The Context Selector:
```rust
GetKlineDataSnafu {
    symbol: String,
    message: String,
    code: Option<MT5ErrorCode>,
    // source/backtrace handled automatically
}
```

### Key differences:

#### 1. Source Error Handling:
```rust
// Manual - Error-prone
http_request().await.map_err(|e| Mt5Error::GetKlineData {
    symbol: "EURUSD".to_string(),
    message: "API failed".to_string(),
    code: None,
    source: e,                           // L Must manually set
    backtrace: Backtrace::generate(),    // L Must manually generate
})

// Context selector - Automatic
http_request().await.context(GetKlineDataSnafu {
    symbol: "EURUSD".to_string(),
    message: "API failed".to_string(),
    code: None,
    //  source/backtrace handled automatically
})
```

#### 2. Error Chain Preservation:
Context selectors automatically preserve the original error as the source, while manual construction requires you to remember to include it.

#### 3. Type Safety:
Context selectors prevent you from forgetting required fields like backtrace generation.

**Summary:** Context selectors provide automatic error chaining, backtrace generation, type safety, and ergonomics that manual error construction cannot match.

---

## 8. Q: Can snafu create different contexts for the same error?

**A:** Yes! This is one of snafu's most powerful features - **contextual error transformation**.

### Example - Same io::Error, different contexts:
```rust
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Unable to read configuration from {}", path.display()))]
    ReadConfiguration { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to write result to {}", path.display()))]
    WriteResult { source: io::Error, path: PathBuf },
}

fn process_data() -> Result<()> {
    // Same underlying io::Error gets different semantic meanings
    let config = fs::read_to_string("config.toml")
        .context(ReadConfigurationSnafu { path: "config.toml" })?;
        
    fs::write("output.txt", "data")  
        .context(WriteResultSnafu { path: "output.txt" })?;
    Ok(())
}
```

### What happens:
- **Step 1**: `io::Error` from read ’ `Error::ReadConfiguration` 
- **Step 2**: `io::Error` from write ’ `Error::WriteResult`

### Why this is powerful:
```rust
match error {
    Error::ReadConfiguration { path, .. } => {
        println!("Config missing at {}, using defaults", path.display());
    }
    Error::WriteResult { path, .. } => {
        println!("Output failed to {}, trying backup", path.display());
    }
}
```

This creates a **semantic error hierarchy** where the same low-level error gets meaningful domain-specific context.

---

## 9. Q: Can thiserror do the same function?

**A:** **No, thiserror cannot do contextual error transformation!** This is a key differentiating feature.

### What thiserror CAN do:
```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),  // Automatic conversion
}
```

### What thiserror CANNOT do:
```rust
// L NOT possible with thiserror
fn process_data() -> Result<(), MyError> {
    let config = fs::read_to_string("config.toml")?;     // Just MyError::Io
    fs::write("output.txt", "data")?;                    // Also just MyError::Io
    // You lose context about WHICH operation failed!
    Ok(())
}
```

### Comparison:

| Feature | thiserror | snafu |
|---------|-----------|--------|
| Basic error definition |  |  |
| Automatic conversions |  `#[from]` |  `From` impls |
| Source error chaining |  `#[source]` |  Automatic |
| **Contextual transformation** | L Manual only |  **Built-in** |
| Error context selectors | L |  **Auto-generated** |
| Ergonomic context addition | L Verbose `map_err` |  **`.context()`** |

### thiserror workaround (verbose):
```rust
fn process() -> Result<(), FileError> {
    let config = fs::read_to_string("config.toml")
        .map_err(|e| FileError::ReadConfig { 
            path: "config.toml".to_string(), 
            source: e 
        })?;  // L Lots of boilerplate
}
```

### snafu approach (clean):
```rust  
fn process() -> Result<(), FileError> {
    let config = fs::read_to_string("config.toml")
        .context(ReadConfigSnafu { path: "config.toml" })?;  //  Clean
}
```

**Conclusion:** The contextual error transformation is snafu's killer feature that thiserror doesn't provide ergonomically.

---

## Summary

Snafu provides powerful error handling capabilities that solve the original problem of losing error context information. The key advantages are:

1. **Preserves error variants** when adding context
2. **Automatic error chaining** and backtrace generation  
3. **Contextual error transformation** - same error type, different semantic meanings
4. **Type-safe context selectors** with clean syntax
5. **Rich error hierarchies** for better error handling

The migration from thiserror to snafu eliminates the issue where `with_context()` was converting all errors to `Internal` variants, while providing much more powerful and ergonomic error handling patterns.