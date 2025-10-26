use crate::backtest_strategy_engine::node::kline_node::KlineNode;
use serde_json::json;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::kline_node_error::KlineNodeError;

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a valid base config for testing
fn create_valid_config() -> serde_json::Value {
    json!({
        "id": "kline_node_1",
        "data": {
            "nodeName": "Kline Node 1",
            "strategyId": 123,
            "backtestConfig": {
                "dataSource": "exchange",
                "exchangeModeConfig": {
                    "selectedAccount": {
                        "accountName": "Test Account",
                        "availableBalance": 10000.0,
                        "exchange": "binance",
                        "id": 1
                    },
                    "selectedSymbols": [
                        {
                            "configId": 1,
                            "interval": "1m",
                            "outputHandleId": "kline_node_1_output_1",
                            "symbol": "BTCUSDT"
                        }
                    ],
                    "timeRange": {
                        "startDate": "2025-10-17 08:00:00 +08:00",
                        "endDate": "2025-10-18 08:00:00 +08:00"
                    }
                }
            }
        }
    })
}

// =============================================================================
// Test Cases: Missing Required Fields
// =============================================================================

#[test]
fn test_missing_id_field() {
    // Test: Missing top-level 'id' field
    // Expected: NodeIdIsNull error
    let mut config = create_valid_config();
    config.as_object_mut().unwrap().remove("id");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'id' field is missing");

    match result.unwrap_err() {
        KlineNodeError::NodeIdIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeIdIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_id_is_null_value() {
    // Test: 'id' field is explicitly null
    // Expected: NodeIdIsNull error
    let mut config = create_valid_config();
    config["id"] = json!(null);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'id' is null");

    match result.unwrap_err() {
        KlineNodeError::NodeIdIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeIdIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_missing_data_field() {
    // Test: Missing top-level 'data' field
    // Expected: NodeDataIsNull error with node_id
    let mut config = create_valid_config();
    config.as_object_mut().unwrap().remove("data");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'data' field is missing");

    match result.unwrap_err() {
        KlineNodeError::NodeDataIsNull { node_id, .. } => {
            assert_eq!(node_id, "kline_node_1", "Error should contain correct node_id");
        }
        other => panic!("Expected NodeDataIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_data_is_null_value() {
    // Test: 'data' field is explicitly null
    // Expected: NodeNameIsNull error (because null.get("nodeName") returns None)
    // Note: JSON null is still a valid JSON value, so .get() doesn't fail, but returns None
    let mut config = create_valid_config();
    config["data"] = json!(null);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'data' is null");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type (first field accessed on null data)
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_missing_node_name_field() {
    // Test: Missing 'nodeName' field in 'data'
    // Expected: NodeNameIsNull error with node_id
    let mut config = create_valid_config();
    config["data"].as_object_mut().unwrap().remove("nodeName");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'nodeName' field is missing");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { node_id, .. } => {
            assert_eq!(node_id, "kline_node_1", "Error should contain correct node_id");
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_node_name_is_null_value() {
    // Test: 'nodeName' field is explicitly null
    // Expected: NodeNameIsNull error
    let mut config = create_valid_config();
    config["data"]["nodeName"] = json!(null);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'nodeName' is null");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_missing_strategy_id_field() {
    // Test: Missing 'strategyId' field in 'data'
    // Expected: ConfigFieldValueNull error with field_name="strategyId"
    let mut config = create_valid_config();
    config["data"].as_object_mut().unwrap().remove("strategyId");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'strategyId' field is missing");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { node_name, field_name, .. } => {
            assert_eq!(node_name, "Kline Node 1", "Error should contain correct node_name");
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_strategy_id_is_null_value() {
    // Test: 'strategyId' field is explicitly null
    // Expected: ConfigFieldValueNull error
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(null);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'strategyId' is null");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_missing_backtest_config_field() {
    // Test: Missing 'backtestConfig' field in 'data'
    // Expected: ConfigFieldValueNull error with field_name="backtestConfig"
    let mut config = create_valid_config();
    config["data"].as_object_mut().unwrap().remove("backtestConfig");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'backtestConfig' field is missing");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { node_name, field_name, .. } => {
            assert_eq!(node_name, "Kline Node 1", "Error should contain correct node_name");
            assert_eq!(field_name, "backtestConfig", "Error should indicate 'backtestConfig' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_backtest_config_is_null_value() {
    // Test: 'backtestConfig' field is explicitly null
    // Expected: ConfigDeserializationFailed error (serde cannot deserialize null to struct)
    let mut config = create_valid_config();
    config["data"]["backtestConfig"] = json!(null);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'backtestConfig' is null");

    match result.unwrap_err() {
        KlineNodeError::ConfigDeserializationFailed { .. } => {
            // Expected error type (serde fails to deserialize null to struct)
        }
        other => panic!("Expected ConfigDeserializationFailed error, got: {:?}", other),
    }
}

// =============================================================================
// Test Cases: Invalid Field Types
// =============================================================================

#[test]
fn test_id_not_string() {
    // Test: 'id' field is a number instead of string
    // Expected: NodeIdIsNull error (because as_str() fails)
    let mut config = create_valid_config();
    config["id"] = json!(123);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'id' is not a string");

    match result.unwrap_err() {
        KlineNodeError::NodeIdIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeIdIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_id_is_boolean() {
    // Test: 'id' field is a boolean
    // Expected: NodeIdIsNull error
    let mut config = create_valid_config();
    config["id"] = json!(true);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'id' is a boolean");

    match result.unwrap_err() {
        KlineNodeError::NodeIdIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeIdIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_id_is_array() {
    // Test: 'id' field is an array
    // Expected: NodeIdIsNull error
    let mut config = create_valid_config();
    config["id"] = json!([1, 2, 3]);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'id' is an array");

    match result.unwrap_err() {
        KlineNodeError::NodeIdIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeIdIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_node_name_not_string() {
    // Test: 'nodeName' field is a number instead of string
    // Expected: NodeNameIsNull error
    let mut config = create_valid_config();
    config["data"]["nodeName"] = json!(456);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'nodeName' is not a string");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_node_name_is_object() {
    // Test: 'nodeName' field is an object
    // Expected: NodeNameIsNull error
    let mut config = create_valid_config();
    config["data"]["nodeName"] = json!({"name": "test"});

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'nodeName' is an object");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_strategy_id_not_number() {
    // Test: 'strategyId' field is a string instead of number
    // Expected: ConfigFieldValueNull error (because as_i64() fails)
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!("123");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'strategyId' is not a number");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_strategy_id_is_float() {
    // Test: 'strategyId' field is a float number
    // Expected: ConfigFieldValueNull error (as_i64() fails for float values in serde_json)
    // Note: serde_json's as_i64() only works for integers, not floats
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(123.0);

    let result = KlineNode::check_kline_node_config(config);

    // This fails because as_i64() doesn't accept float representations
    assert!(result.is_err(), "Should fail when 'strategyId' is 123.0 (float)");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_strategy_id_is_float_with_decimal() {
    // Test: 'strategyId' field is a float with decimal part
    // Expected: ConfigFieldValueNull error (as_i64() fails for 123.45)
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(123.45);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'strategyId' has decimal part");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_backtest_config_invalid_structure() {
    // Test: 'backtestConfig' has invalid structure (not matching KlineNodeBacktestConfig)
    // Expected: ConfigDeserializationFailed error
    let mut config = create_valid_config();
    config["data"]["backtestConfig"] = json!({
        "invalidField": "some value"
    });

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'backtestConfig' structure is invalid");

    match result.unwrap_err() {
        KlineNodeError::ConfigDeserializationFailed { node_name, .. } => {
            assert_eq!(node_name, "Kline Node 1", "Error should contain correct node_name");
        }
        other => panic!("Expected ConfigDeserializationFailed error, got: {:?}", other),
    }
}

#[test]
fn test_backtest_config_missing_datasource() {
    // Test: 'backtestConfig' missing required 'dataSource' field
    // Expected: ConfigDeserializationFailed error
    let mut config = create_valid_config();
    config["data"]["backtestConfig"] = json!({
        "exchangeModeConfig": {
            "selectedAccount": {
                "accountName": "Test",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
            },
            "selectedSymbols": [],
            "timeRange": {
                "startDate": "2025-10-17 08:00:00 +08:00",
                "endDate": "2025-10-18 08:00:00 +08:00"
            }
        }
    });

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'dataSource' is missing");

    match result.unwrap_err() {
        KlineNodeError::ConfigDeserializationFailed { .. } => {
            // Expected error type
        }
        other => panic!("Expected ConfigDeserializationFailed error, got: {:?}", other),
    }
}

// =============================================================================
// Test Cases: Valid Configurations
// =============================================================================

#[test]
fn test_valid_complete_config() {
    // Test: Complete and valid configuration
    // Expected: Success with all fields correctly parsed
    let config = create_valid_config();

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with valid complete config");

    let (strategy_id, node_id, node_name, backtest_config) = result.unwrap();

    // Verify all returned values
    assert_eq!(strategy_id, 123, "Strategy ID should be 123");
    assert_eq!(node_id, "kline_node_1", "Node ID should be 'kline_node_1'");
    assert_eq!(node_name, "Kline Node 1", "Node name should be 'Kline Node 1'");
    assert_eq!(
        format!("{:?}", backtest_config.data_source),
        "Exchange",
        "Data source should be Exchange"
    );
}

#[test]
fn test_valid_config_with_minimal_backtest_config() {
    // Test: Minimal valid configuration (backtestConfig with only required fields)
    // Expected: Success
    let config = json!({
        "id": "kline_node_2",
        "data": {
            "nodeName": "Minimal Kline Node",
            "strategyId": 456,
            "backtestConfig": {
                "dataSource": "exchange"
                // exchangeModeConfig is optional (Option type)
            }
        }
    });

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with minimal valid config");

    let (strategy_id, node_id, node_name, backtest_config) = result.unwrap();

    assert_eq!(strategy_id, 456, "Strategy ID should be 456");
    assert_eq!(node_id, "kline_node_2", "Node ID should be 'kline_node_2'");
    assert_eq!(node_name, "Minimal Kline Node", "Node name should be 'Minimal Kline Node'");
    assert!(
        backtest_config.exchange_mode_config.is_none(),
        "Exchange mode config should be None"
    );
}

// =============================================================================
// Test Cases: Edge Cases
// =============================================================================

#[test]
fn test_empty_id_string() {
    // Test: 'id' is an empty string
    // Expected: Success (empty string is still a valid string type)
    let mut config = create_valid_config();
    config["id"] = json!("");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed when 'id' is empty string");

    let (_, node_id, _, _) = result.unwrap();
    assert_eq!(node_id, "", "Node ID should be empty string");
}

#[test]
fn test_empty_node_name_string() {
    // Test: 'nodeName' is an empty string
    // Expected: Success (empty string is still a valid string type)
    let mut config = create_valid_config();
    config["data"]["nodeName"] = json!("");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed when 'nodeName' is empty string");

    let (_, _, node_name, _) = result.unwrap();
    assert_eq!(node_name, "", "Node name should be empty string");
}

#[test]
fn test_strategy_id_zero() {
    // Test: 'strategyId' is 0
    // Expected: Success (0 is a valid number)
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(0);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed when 'strategyId' is 0");

    let (strategy_id, _, _, _) = result.unwrap();
    assert_eq!(strategy_id, 0, "Strategy ID should be 0");
}

#[test]
fn test_strategy_id_negative() {
    // Test: 'strategyId' is negative
    // Expected: Success (will be cast to u32, behavior depends on platform)
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(-1);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed when 'strategyId' is negative");

    // Note: The actual value after casting i64 to u32 may be platform-dependent
    // We just verify it parses successfully
    let (strategy_id, _, _, _) = result.unwrap();
    println!("Negative strategy_id (-1) cast to u32: {}", strategy_id);
}

#[test]
fn test_strategy_id_max_value() {
    // Test: 'strategyId' is i64 maximum value
    // Expected: Success
    let mut config = create_valid_config();
    config["data"]["strategyId"] = json!(i64::MAX);

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed when 'strategyId' is i64::MAX");

    let (strategy_id, _, _, _) = result.unwrap();
    // When cast to u32, this will overflow
    println!("i64::MAX cast to u32: {}", strategy_id);
}

#[test]
fn test_very_long_id_string() {
    // Test: 'id' is a very long string (1000+ characters)
    // Expected: Success (string length is not restricted)
    let mut config = create_valid_config();
    let long_id = "a".repeat(1000);
    config["id"] = json!(long_id.clone());

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with very long 'id' string");

    let (_, node_id, _, _) = result.unwrap();
    assert_eq!(node_id.len(), 1000, "Node ID should have 1000 characters");
    assert_eq!(node_id, long_id, "Node ID should match the long string");
}

#[test]
fn test_very_long_node_name_string() {
    // Test: 'nodeName' is a very long string
    // Expected: Success
    let mut config = create_valid_config();
    let long_name = "Node ".to_string() + &"X".repeat(1000);
    config["data"]["nodeName"] = json!(long_name.clone());

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with very long 'nodeName' string");

    let (_, _, node_name, _) = result.unwrap();
    assert_eq!(node_name, long_name, "Node name should match the long string");
}

#[test]
fn test_id_with_special_characters() {
    // Test: 'id' contains special characters
    // Expected: Success (special characters are allowed in strings)
    let mut config = create_valid_config();
    config["id"] = json!("node-id_123@test!#$%");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with special characters in 'id'");

    let (_, node_id, _, _) = result.unwrap();
    assert_eq!(node_id, "node-id_123@test!#$%", "Node ID should contain special characters");
}

#[test]
fn test_node_name_with_unicode() {
    // Test: 'nodeName' contains Unicode characters (emoji and special chars)
    // Expected: Success (Unicode is valid in JSON strings)
    let mut config = create_valid_config();
    config["data"]["nodeName"] = json!("Kline_Node_🚀_测试");

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_ok(), "Should succeed with Unicode in 'nodeName'");

    let (_, _, node_name, _) = result.unwrap();
    assert_eq!(node_name, "Kline_Node_🚀_测试", "Node name should contain Unicode characters");
}

// =============================================================================
// Test Cases: Object Structure Tests
// =============================================================================

#[test]
fn test_data_is_empty_object() {
    // Test: 'data' is an empty object {}
    // Expected: NodeNameIsNull error (first missing field)
    let mut config = create_valid_config();
    config["data"] = json!({});

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'data' is empty object");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type (first field that's checked)
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

#[test]
fn test_data_has_only_node_name() {
    // Test: 'data' only has 'nodeName', missing other required fields
    // Expected: ConfigFieldValueNull error for 'strategyId'
    let mut config = create_valid_config();
    config["data"] = json!({
        "nodeName": "Test Node"
    });

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when only 'nodeName' is present");

    match result.unwrap_err() {
        KlineNodeError::ConfigFieldValueNull { field_name, .. } => {
            assert_eq!(field_name, "strategyId", "Error should indicate 'strategyId' field");
        }
        other => panic!("Expected ConfigFieldValueNull error, got: {:?}", other),
    }
}

#[test]
fn test_data_has_only_strategy_id() {
    // Test: 'data' only has 'strategyId', missing 'nodeName'
    // Expected: NodeNameIsNull error
    let mut config = create_valid_config();
    config["data"] = json!({
        "strategyId": 123
    });

    let result = KlineNode::check_kline_node_config(config);

    assert!(result.is_err(), "Should fail when 'nodeName' is missing");

    match result.unwrap_err() {
        KlineNodeError::NodeNameIsNull { .. } => {
            // Expected error type
        }
        other => panic!("Expected NodeNameIsNull error, got: {:?}", other),
    }
}

// =============================================================================
// Test Cases: Extra Fields
// =============================================================================

#[test]
fn test_config_with_extra_fields() {
    // Test: Configuration contains extra undefined fields
    // Expected: Success (serde ignores extra fields by default)
    let mut config = create_valid_config();
    config["extraField1"] = json!("extra value");
    config["data"]["extraField2"] = json!(999);
    config["data"]["backtestConfig"]["extraField3"] = json!(true);

    let result = KlineNode::check_kline_node_config(config);

    assert!(
        result.is_ok(),
        "Should succeed when config has extra fields (serde ignores them)"
    );

    let (strategy_id, node_id, node_name, _) = result.unwrap();
    assert_eq!(strategy_id, 123, "Strategy ID should be correctly parsed");
    assert_eq!(node_id, "kline_node_1", "Node ID should be correctly parsed");
    assert_eq!(node_name, "Kline Node 1", "Node name should be correctly parsed");
}
