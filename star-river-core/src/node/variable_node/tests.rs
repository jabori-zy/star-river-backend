#[cfg(test)]
mod tests {
    use crate::node::variable_node::variable_config::*;
    use crate::node::variable_node::VariableConfig;
    use serde_json;

    #[test]
    fn test_deserialize_variable_configs() {
        let json = r#"
        [
          {
            "configId": 1,
            "inputHandleId": "variable_node_3_input_1",
            "outputHandleId": "variable_node_3_output_1",
            "symbol": null,
            "triggerConfig": {
              "config": {
                "interval": 1,
                "mode": "interval",
                "unit": "hour"
              },
              "type": "timer"
            },
            "varDisplayName": "数字测试11",
            "varName": "number_test",
            "varOperation": "get",
            "varType": "custom",
            "varValue": 0,
            "varValueType": "number"
          },
          {
            "configId": 2,
            "inputHandleId": "variable_node_3_input_2",
            "outputHandleId": "variable_node_3_output_2",
            "triggerConfig": {
              "config": {
                "interval": 1,
                "mode": "interval",
                "unit": "hour"
              },
              "type": "timer"
            },
            "updateOperationValue": 123123,
            "updateVarValueOperation": "set",
            "varDisplayName": "数字测试1",
            "varName": "number_test",
            "varOperation": "update",
            "varType": "custom",
            "varValueType": "number"
          },
          {
            "configId": 3,
            "inputHandleId": "variable_node_3_input_3",
            "outputHandleId": "variable_node_3_output_3",
            "triggerConfig": {
              "config": {
                "interval": 1,
                "mode": "interval",
                "unit": "hour"
              },
              "type": "timer"
            },
            "varDisplayName": "数字测试1",
            "varInitialValue": 0.005,
            "varName": "number_test",
            "varOperation": "reset",
            "varType": "custom",
            "varValueType": "number"
          }
        ]
        "#;

        let configs: Result<Vec<VariableConfig>, _> = serde_json::from_str(json);

        assert!(configs.is_ok(), "反序列化失败: {:?}", configs.err());
        println!("configs: {:#?}", configs);

        let configs = configs.unwrap();
        assert_eq!(configs.len(), 3);

        // 验证第一个配置是 GetConfig
        assert!(configs[0].is_get(), "第一个配置应该是 GetConfig");
        match &configs[0] {
            VariableConfig::Get(get_config) => {
                assert_eq!(get_config.config_id(), 1);
                assert_eq!(get_config.var_name(), "number_test");
                // assert_eq!(get_config.var_display_name, "数字测试11");
            }
            _ => panic!("第一个配置类型错误"),
        }

        // 验证第二个配置是 UpdateConfig
        assert!(configs[1].is_update(), "第二个配置应该是 UpdateConfig");
        match &configs[1] {
            VariableConfig::Update(update_config) => {
                assert_eq!(update_config.config_id(), 2);
                assert_eq!(update_config.var_name(), "number_test");
                assert!(update_config.update_operation_value().is_some());
            }
            _ => panic!("第二个配置类型错误"),
        }

        // 验证第三个配置是 ResetConfig
        assert!(configs[2].is_reset(), "第三个配置应该是 ResetConfig");
        match &configs[2] {
            VariableConfig::Reset(reset_config) => {
                assert_eq!(reset_config.config_id(), 3);
                // assert_eq!(reset_config.var_name(), "number_test");
            }
            _ => panic!("第三个配置类型错误"),
        }

        println!("✅ 所有配置都正确反序列化！");
    }

    #[test]
    fn test_deserialize_get_variable_config() {
        let json = r#"
        {
          "configId": 1,
          "inputHandleId": "input_1",
          "outputHandleId": "output_1",
          "varType": "custom",
          "varName": "test_var",
          "varDisplayName": "测试变量",
          "varValueType": "number",
          "triggerConfig": null,
          "varOperation": "get",
          "varValue": 100.5
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "GetConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_get());

        println!("✅ GetConfig 反序列化成功！");
    }

    #[test]
    fn test_deserialize_update_variable_config() {
        let json = r#"
        {
          "configId": 2,
          "inputHandleId": "input_2",
          "outputHandleId": "output_2",
          "varType": "custom",
          "varName": "test_var",
          "varDisplayName": "测试变量",
          "varValueType": "number",
          "triggerConfig": null,
          "varOperation": "update",
          "updateVarValueOperation": "set",
          "updateOperationValue": 200
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "UpdateConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_update());

        println!("✅ UpdateConfig 反序列化成功！");
    }

    #[test]
    fn test_deserialize_reset_variable_config() {
        let json = r#"
        {
          "configId": 3,
          "inputHandleId": "input_3",
          "outputHandleId": "output_3",
          "varType": "custom",
          "varName": "test_var",
          "varDisplayName": "测试变量",
          "varValueType": "number",
          "triggerConfig": null,
          "varOperation": "reset",
          "varInitialValue": 0.005
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "ResetConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_reset());

        println!("✅ ResetConfig 反序列化成功！");
    }

    #[test]
    fn test_deserialize_backtest_config() {
        let json = r#"
        {
          "backtestConfig": {
            "dataSource": "exchange",
            "exchangeModeConfig": {
              "selectedAccount": {
                "accountName": "账号1",
                "availableBalance": 0,
                "exchange": "binance",
                "id": 1
              }
            },
            "variableConfigs": [
              {
                "configId": 1,
                "inputHandleId": "variable_node_3_input_1",
                "outputHandleId": "variable_node_3_output_1",
                "symbol": null,
                "triggerConfig": {
                  "config": {
                    "interval": 1,
                    "mode": "interval",
                    "unit": "hour"
                  },
                  "type": "timer"
                },
                "varDisplayName": "数字测试11",
                "varName": "number_test",
                "varOperation": "get",
                "varType": "custom",
                "varValue": 0,
                "varValueType": "number"
              },
              {
                "configId": 2,
                "inputHandleId": "variable_node_3_input_2",
                "outputHandleId": "variable_node_3_output_2",
                "triggerConfig": {
                  "config": {
                    "interval": 1,
                    "mode": "interval",
                    "unit": "hour"
                  },
                  "type": "timer"
                },
                "updateOperationValue": 123123,
                "updateVarValueOperation": "set",
                "varDisplayName": "数字测试1",
                "varName": "number_test",
                "varOperation": "update",
                "varType": "custom",
                "varValueType": "number"
              },
              {
                "configId": 3,
                "inputHandleId": "variable_node_3_input_3",
                "outputHandleId": "variable_node_3_output_3",
                "triggerConfig": {
                  "config": {
                    "interval": 1,
                    "mode": "interval",
                    "unit": "hour"
                  },
                  "type": "timer"
                },
                "varDisplayName": "数字测试1",
                "varInitialValue": 0.005,
                "varName": "number_test",
                "varOperation": "reset",
                "varType": "custom",
                "varValueType": "number"
              }
            ]
          },
          "nodeName": "变量节点3",
          "strategyId": 2
        }
        "#;

        use crate::node::variable_node::VariableNodeBacktestConfig;

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TestNodeData {
            strategy_id: i32,
            node_name: String,
            backtest_config: Option<VariableNodeBacktestConfig>,
        }

        let node_data: Result<TestNodeData, _> = serde_json::from_str(json);
        assert!(node_data.is_ok(), "完整配置反序列化失败: {:?}", node_data.err());

        let node_data = node_data.unwrap();
        assert_eq!(node_data.strategy_id, 2);
        assert_eq!(node_data.node_name, "变量节点3");
        assert!(node_data.backtest_config.is_some());

        let backtest_config = node_data.backtest_config.unwrap();
        assert_eq!(backtest_config.variable_configs.len(), 3);

        println!("✅ 完整的回测配置反序列化成功！");
        println!("   - 配置数量: {}", backtest_config.variable_configs.len());
        println!("   - Get配置: {}", backtest_config.variable_configs[0].is_get());
        println!("   - Update配置: {}", backtest_config.variable_configs[1].is_update());
        println!("   - Reset配置: {}", backtest_config.variable_configs[2].is_reset());
    }
}
