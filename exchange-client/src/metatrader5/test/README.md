# MetaTrader5 Module Tests

This directory contains comprehensive unit tests for the MetaTrader5 exchange client module.

## Test Structure

### `metatrader5_tests.rs`
Tests for the core MetaTrader5 struct functionality:
- **Constructor tests**: Verify proper initialization of MetaTrader5 instances
- **Process name generation**: Test unique process name creation for different terminal IDs  
- **Port assignment**: Test automatic port assignment logic (8000 + terminal_id)
- **HTTP client creation**: Test MT5 HTTP client initialization
- **Clone and Debug traits**: Test derived trait implementations

### `exchange_client_tests.rs`
Tests for the ExchangeClient trait implementation:
- **Trait method tests**: Test all ExchangeClient methods
- **Type conversion tests**: Test downcasting and type safety
- **Exchange type identification**: Test proper exchange type reporting
- **Error handling**: Test error cases when HTTP client is not initialized
- **WebSocket functionality**: Test WebSocket connection and streaming methods
- **Market data methods**: Test K-line data retrieval and processing
- **Trading operations**: Test order creation, position management, etc.

### `helper_tests.rs`  
Tests for helper functions and utilities:
- **Error type tests**: Test MetaTrader5Error enum and conversions
- **Path creation tests**: Test executable file path generation
- **Resource handling**: Test embedded resource extraction logic
- **Edge case handling**: Test invalid inputs and boundary conditions
- **Directory management**: Test terminal-specific directory creation

## Test Patterns

### Mocking Strategy
Since MetaTrader5 involves external processes and network connections, tests focus on:
- **Unit-level testing**: Test individual methods in isolation
- **Error path testing**: Test error conditions without external dependencies  
- **State verification**: Test internal state changes
- **Input validation**: Test parameter validation and edge cases

### Async Testing
Uses `#[tokio::test]` for async methods, ensuring proper async runtime handling.

### No External Dependencies
Tests are designed to run without requiring:
- Actual MT5 server processes
- Network connections  
- Windows-specific system calls
- Embedded executable resources

## Running Tests

```bash
# Run all MetaTrader5 tests
cargo test --package exchange-client metatrader5

# Run specific test modules
cargo test --package exchange-client metatrader5_tests
cargo test --package exchange-client exchange_client_tests
cargo test --package exchange-client helper_tests

# Run a specific test function
cargo test --package exchange-client test_metatrader5_new
cargo test --package exchange-client test_exchange_type

# Run with output
cargo test --package exchange-client metatrader5 -- --nocapture
```

## Test Coverage

The tests cover:
- ✅ Constructor and basic initialization
- ✅ ExchangeClient trait implementation
- ✅ Error handling and edge cases
- ✅ Type safety and conversions
- ✅ Basic functionality without external dependencies
- ❌ Integration testing with actual MT5 processes (requires separate integration test suite)
- ❌ Network communication testing (requires mock servers)
- ❌ Windows-specific process management (requires Windows environment)

## Future Improvements

1. **Integration Tests**: Add separate integration test suite for end-to-end testing
2. **Mock Framework**: Integrate mockall or similar for more sophisticated mocking
3. **Property-based Testing**: Use proptest for generating test cases
4. **Performance Testing**: Add benchmarks for critical paths
5. **Windows-specific Tests**: Add conditional compilation for Windows-only functionality