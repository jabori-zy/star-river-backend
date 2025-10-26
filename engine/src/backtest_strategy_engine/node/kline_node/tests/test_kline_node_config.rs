

use crate::backtest_strategy_engine::node::kline_node::kline_node_type::KlineNodeBacktestConfig;
use serde_json::json;

#[test]
fn test_config_without_datasource() {
    // Test missing dataSource field
    let config_json = json!({
        "exchangeModeConfig": {
            "selectedAccount": {
                "accountName": "Account 1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
            },
            "selectedSymbols": [
                {
                    "configId": 1,
                    "interval": "1m",
                    "outputHandleId": "kline_node_2_output_1",
                    "symbol": "BTCUSDT"
                }
            ],
            "timeRange": {
                "endDate": "2025-10-18 08:00:00 +08:00",
                "startDate": "2025-10-17 08:00:00 +08:00"
            }
        }
    });

    let result = serde_json::from_value::<KlineNodeBacktestConfig>(config_json);
    assert!(result.is_err(), "Should fail due to missing dataSource");

    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("dataSource") || error_message.contains("missing field"),
        "Error message should indicate missing dataSource field: {}",
        error_message
    );
}

#[test]
fn test_config_without_exchange_mode_config() {
    // Test missing exchangeModeConfig field
    // Note: According to type definition, exchangeModeConfig is Option type, so this test should succeed
    let config_json = json!({
        "dataSource": "exchange"
    });

    let result = serde_json::from_value::<KlineNodeBacktestConfig>(config_json);
    assert!(result.is_ok(), "exchangeModeConfig is Option type, can be None");

    let config = result.unwrap();
    assert!(config.exchange_mode_config.is_none(), "exchangeModeConfig should be None");
}

#[test]
fn test_config_without_symbol() {
    // Test selectedSymbols as empty array
    let config_json = json!({
        "dataSource": "exchange",
        "exchangeModeConfig": {
            "selectedAccount": {
                "accountName": "Account 1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
            },
            "selectedSymbols": [],
            "timeRange": {
                "endDate": "2025-10-18 08:00:00 +08:00",
                "startDate": "2025-10-17 08:00:00 +08:00"
            }
        }
    });

    let result = serde_json::from_value::<KlineNodeBacktestConfig>(config_json);
    assert!(result.is_ok(), "selectedSymbols can be empty array");

    let config = result.unwrap();
    assert_eq!(
        config.exchange_mode_config.as_ref().unwrap().selected_symbols.len(),
        0,
        "selectedSymbols should be empty"
    );
}

#[test]
fn test_config_without_time_range() {
    // Test missing timeRange field
    let config_json = json!({
        "dataSource": "exchange",
        "exchangeModeConfig": {
            "selectedAccount": {
                "accountName": "Account 1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
            },
            "selectedSymbols": [
                {
                    "configId": 1,
                    "interval": "1m",
                    "outputHandleId": "kline_node_2_output_1",
                    "symbol": "BTCUSDT"
                }
            ]
        }
    });

    let result = serde_json::from_value::<KlineNodeBacktestConfig>(config_json);
    assert!(result.is_err(), "Should fail due to missing timeRange");

    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("timeRange") || error_message.contains("missing field"),
        "Error message should indicate missing timeRange field: {}",
        error_message
    );
}

#[test]
fn test_correct_config() {
    // Test correct configuration
    let config_json = json!({
        "dataSource": "exchange",
        "exchangeModeConfig": {
            "selectedAccount": {
                "accountName": "Account 1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
            },
            "selectedSymbols": [
                {
                    "configId": 1,
                    "interval": "1m",
                    "outputHandleId": "kline_node_2_output_1",
                    "symbol": "BTCUSDT"
                }
            ],
            "timeRange": {
                "endDate": "2025-10-18 08:00:00 +08:00",
                "startDate": "2025-10-17 08:00:00 +08:00"
            }
        }
    });

    let result = serde_json::from_value::<KlineNodeBacktestConfig>(config_json);
    assert!(result.is_ok(), "Correct config should parse successfully");

    let config = result.unwrap();

    // Verify each field
    assert_eq!(
        format!("{:?}", config.data_source),
        "Exchange",
        "dataSource should be Exchange"
    );

    let exchange_config = config.exchange_mode_config.as_ref().unwrap();
    assert_eq!(
        format!("{:?}", exchange_config.selected_account.exchange),
        "Binance",
        "exchange should be Binance"
    );
    assert_eq!(
        exchange_config.selected_account.account_name,
        "Account 1",
        "accountName should be 'Account 1'"
    );
    assert_eq!(
        exchange_config.selected_symbols.len(),
        1,
        "Should have 1 symbol"
    );

    let symbol = &exchange_config.selected_symbols[0];
    assert_eq!(symbol.symbol, "BTCUSDT", "symbol should be BTCUSDT");
    assert_eq!(
        format!("{:?}", symbol.interval),
        "Minutes1",
        "interval should be 1m"
    );
    assert_eq!(
        symbol.output_handle_id,
        "kline_node_2_output_1",
        "outputHandleId should be kline_node_2_output_1"
    );
    assert_eq!(symbol.config_id, 1, "configId should be 1");

    // Verify time range
    assert!(
        exchange_config.time_range.start_date < exchange_config.time_range.end_date,
        "Start date should be less than end date"
    );
}

