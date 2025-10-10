use crate::binance::Binance;
use crate::exchange_trait::ExchangeSymbolExt;
use ordered_float::OrderedFloat;
use star_river_core::market::Exchange;

fn init_logger() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
}

#[tokio::test]
async fn test_get_symbol() {
    init_logger();
    let exchange = Binance::new();

    let result = exchange.get_symbol("BTCUSDT".to_string()).await;

    assert!(result.is_ok(), "Get symbol should succeed");
    let symbol = result.unwrap();

    // Verify Symbol fields
    assert_eq!(symbol.name, "BTCUSDT", "Symbol name should be BTCUSDT");
    assert_eq!(symbol.exchange, Exchange::Binance, "Exchange should be Binance");
    assert!(symbol.point > OrderedFloat::from(0.0), "Point should be positive");

    println!("Symbol: {:?}", symbol);
}

#[tokio::test]
async fn test_get_symbol_eth() {
    init_logger();
    let exchange = Binance::new();

    let result = exchange.get_symbol("ETHUSDT".to_string()).await;

    assert!(result.is_ok(), "Get symbol ETHUSDT should succeed");
    let symbol = result.unwrap();

    assert_eq!(symbol.name, "ETHUSDT");
    assert_eq!(symbol.exchange, Exchange::Binance);

    println!("ETHUSDT Symbol: {:?}", symbol);
}

#[tokio::test]
async fn test_get_symbol_list() {
    init_logger();
    let exchange = Binance::new();

    let result = exchange.get_symbol_list().await;

    assert!(result.is_ok(), "Get symbol list should succeed");
    let symbols = result.unwrap();

    // Verify returned data is not empty
    assert!(!symbols.is_empty(), "Symbol list should not be empty");

    // Verify first Symbol fields
    let first_symbol = &symbols[0];
    assert!(!first_symbol.name.is_empty(), "Symbol name should not be empty");
    assert!(first_symbol.base.is_some(), "Base asset should exist");
    assert!(first_symbol.quote.is_some(), "Quote asset should exist");
    assert_eq!(first_symbol.exchange, Exchange::Binance, "Exchange should be Binance");

    println!("Total symbols: {}", symbols.len());
    println!("First 5 symbols:");
    for symbol in symbols.iter().take(5) {
        println!("  - {} (base: {:?}, quote: {:?})",
            symbol.name,
            symbol.base,
            symbol.quote
        );
    }
}

#[tokio::test]
async fn test_get_symbol_invalid() {
    init_logger();
    let exchange = Binance::new();

    let result = exchange.get_symbol("INVALIDXYZ123".to_string()).await;

    // Should return error because this symbol does not exist
    assert!(result.is_err(), "Invalid symbol should return error");

    if let Err(e) = result {
        println!("Expected error for invalid symbol: {:?}", e);
    }
}

#[tokio::test]
async fn test_get_symbol_list_verify_popular_symbols() {
    init_logger();
    let exchange = Binance::new();

    let result = exchange.get_symbol_list().await;
    assert!(result.is_ok());
    let symbols = result.unwrap();

    // Verify contains common trading pairs
    let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

    let popular_symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
    for popular in popular_symbols {
        assert!(
            symbol_names.contains(&popular.to_string()),
            "Symbol list should contain {}",
            popular
        );
    }

    println!("Verified popular symbols exist in the list");
}

#[tokio::test]
async fn test_get_symbol_multiple() {
    init_logger();
    let exchange = Binance::new();

    let test_symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

    for symbol_name in test_symbols {
        let result = exchange.get_symbol(symbol_name.to_string()).await;
        assert!(result.is_ok(), "Get symbol {} should succeed", symbol_name);

        let symbol = result.unwrap();
        assert_eq!(symbol.name, symbol_name);
        assert_eq!(symbol.exchange, Exchange::Binance);

        println!("✓ {}: {:?}", symbol_name, symbol);
    }
}

#[tokio::test]
async fn test_get_support_kline_intervals() {
    init_logger();
    let exchange = Binance::new();

    let intervals = exchange.get_support_kline_intervals();

    assert!(!intervals.is_empty(), "Support kline intervals should not be empty");

    // Verify contains common time intervals
    use star_river_core::market::KlineInterval;
    assert!(intervals.contains(&KlineInterval::Minutes1));
    assert!(intervals.contains(&KlineInterval::Minutes5));
    assert!(intervals.contains(&KlineInterval::Minutes15));
    assert!(intervals.contains(&KlineInterval::Hours1));
    assert!(intervals.contains(&KlineInterval::Days1));

    println!("Supported intervals count: {}", intervals.len());
    println!("Supported intervals: {:?}", intervals);
}

#[tokio::test]
async fn test_get_symbol_case_sensitive() {
    init_logger();
    let exchange = Binance::new();

    // Binance trading pairs are usually uppercase
    let result_upper = exchange.get_symbol("BTCUSDT".to_string()).await;
    let result_lower = exchange.get_symbol("btcusdt".to_string()).await;

    // Uppercase should succeed
    assert!(result_upper.is_ok(), "Uppercase symbol should succeed");

    // Lowercase may fail (depends on API implementation)
    if result_lower.is_ok() {
        println!("API accepts lowercase symbols");
    } else {
        println!("API only accepts uppercase symbols");
    }
}

