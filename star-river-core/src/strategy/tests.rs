#[cfg(test)]
mod tests {
    use crate::strategy::*;
    use rust_decimal::Decimal;
    use serde_json;

    #[test]
    fn test_deserialize_backtest_strategy_config() {
        let json = r#"
        {
          "customVariables": [
            {
              "initialValue": 0.005,
              "varDisplayName": "数字测试1",
              "varName": "number_test",
              "varValue": 0.005,
              "varValueType": "number"
            },
            {
              "initialValue": "阿斯顿发顺丰的",
              "varDisplayName": "字符串测试",
              "varName": "string_test",
              "varValue": "阿斯顿发顺丰的",
              "varValueType": "string"
            },
            {
              "varName": "percent_test",
              "varDisplayName": "百分比测试",
              "varValueType": "percentage",
              "initialValue": 123123,
              "varValue": 123123
            }
          ],
          "dataSource": "exchange",
          "exchangeModeConfig": {
            "selectedAccounts": [
              {
                "accountName": "账号1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
              }
            ],
            "timeRange": {
              "endDate": "2025-10-18 08:00:00 +08:00",
              "startDate": "2025-10-17 08:00:00 +08:00"
            }
          },
          "feeRate": 0.001,
          "fileModeConfig": null,
          "initialBalance": 10000,
          "leverage": 1,
          "playSpeed": 50
        }
        "#;

        let result: Result<BacktestStrategyConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "反序列化失败: {:?}", result.err());

        let config = result.unwrap();
        println!("✅ BacktestStrategyConfig 反序列化成功！");
        println!("配置详情: {:#?}", config);
    }

    #[test]
    fn test_deserialize_all_variable_types() {
        let json = r#"
        {
          "customVariables": [
            {
              "initialValue": 123.456,
              "varDisplayName": "数字变量",
              "varName": "number_var",
              "varValue": 123.456,
              "varValueType": "number"
            },
            {
              "initialValue": "测试字符串",
              "varDisplayName": "字符串变量",
              "varName": "string_var",
              "varValue": "测试字符串",
              "varValueType": "string"
            },
            {
              "initialValue": true,
              "varDisplayName": "布尔变量",
              "varName": "boolean_var",
              "varValue": true,
              "varValueType": "boolean"
            },
            {
              "initialValue": "2025-10-19 20:02:00 +08:00",
              "varDisplayName": "时间变量",
              "varName": "time_var",
              "varValue": "2025-10-19 20:02:00 +08:00",
              "varValueType": "time"
            },
            {
              "initialValue": 0.85,
              "varDisplayName": "百分比变量",
              "varName": "percentage_var",
              "varValue": 0.85,
              "varValueType": "percentage"
            }
          ],
          "dataSource": "file",
          "exchangeModeConfig": null,
          "feeRate": 0.0001,
          "initialBalance": 5000,
          "leverage": 5,
          "playSpeed": 100
        }
        "#;

        let result: Result<BacktestStrategyConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "反序列化失败: {:?}", result.err());

        let config = result.unwrap();
        println!("✅ 所有变量类型反序列化成功！");

        assert_eq!(config.custom_variables.len(), 5);

        // 验证数字类型
        if let custom_variable::VariableValue::Number(val) = config.custom_variables[0].var_value {
            assert_eq!(val, Decimal::from(123456));
        } else {
            panic!("Expected Number variant");
        }

        // 验证字符串类型
        if let custom_variable::VariableValue::String(val) = &config.custom_variables[1].var_value {
            assert_eq!(val, "测试字符串");
        } else {
            panic!("Expected String variant");
        }

        // 验证布尔类型
        if let custom_variable::VariableValue::Boolean(val) = config.custom_variables[2].var_value {
            assert!(val);
        } else {
            panic!("Expected Boolean variant");
        }

        // 验证时间类型
        assert!(matches!(
            config.custom_variables[3].var_value,
            custom_variable::VariableValue::Time(_)
        ));

        // 验证百分比类型
        if let custom_variable::VariableValue::Percentage(val) = config.custom_variables[4].var_value {
            assert_eq!(val, Decimal::from(85));
        } else {
            panic!("Expected Percentage variant");
        }

        println!("✅ 所有变量类型验证通过！");
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use custom_variable::{CustomVariable, VariableValue};

        // 创建一个完整的配置
        let config = BacktestStrategyConfig {
            data_source: BacktestDataSource::File,
            exchange_mode_config: None,
            initial_balance: 10000.0,
            leverage: 10,
            fee_rate: 0.0001,
            play_speed: 1,
            custom_variables: vec![CustomVariable {
                var_name: "test_var".to_string(),
                var_display_name: "测试变量".to_string(),
                initial_value: VariableValue::Number(Decimal::from(100)),
                var_value: VariableValue::Number(Decimal::from(100)),
            }],
        };

        // 序列化
        let json = serde_json::to_string_pretty(&config).unwrap();
        println!("序列化结果:\n{}", json);

        // 反序列化
        let result: Result<BacktestStrategyConfig, _> = serde_json::from_str(&json);
        assert!(result.is_ok(), "往返测试失败: {:?}", result.err());

        let deserialized_config = result.unwrap();
        assert_eq!(deserialized_config.custom_variables.len(), 1);
        assert_eq!(deserialized_config.custom_variables[0].var_name, "test_var");

        println!("✅ 序列化和反序列化往返测试成功！");
    }
}
