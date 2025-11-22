#[cfg(test)]
mod tests {
    use serde_json;

    use crate::node_infra::variable_node::{trigger::TriggerConfig, variable_config::VariableConfig};

    #[test]
    fn test_deserialize_variable_configs() {
        let json = r#"
        [
          {
            "configId": 1,
            "inputHandleId": "variable_node_6_input_1",
            "outputHandleId": "variable_node_6_output_1",
            "triggerConfig": {
              "config": {
                "errorPolicy": {
                  "expired": {
                    "errorLog": {
                      "notify": false
                    },
                    "strategy": "skip"
                  },
                  "nullValue": {
                    "errorLog": {
                      "notify": false
                    },
                    "replaceValue": -12312313,
                    "strategy": "valueReplace"
                  },
                  "zeroValue": {
                    "errorLog": {
                      "notify": false
                    },
                    "strategy": "skip"
                  }
                },
                "expireDuration": {
                  "duration": 1,
                  "unit": "hour"
                },
                "fromHandleId": "variable_node_5_output_4",
                "fromNodeId": "variable_node_5",
                "fromNodeName": "变量节点5",
                "fromNodeType": "variableNode",
                "fromVar": "percent_test",
                "fromVarConfigId": 4,
                "fromVarDisplayName": "百分比测试4",
                "fromVarValueType": "percentage"
              },
              "type": "dataflow"
            },
            "updateOperationValue": null,
            "updateVarValueOperation": "add",
            "varDisplayName": "百分比测试",
            "varName": "percent_test",
            "varOperation": "update",
            "varType": "custom",
            "varValueType": "percentage"
          },
          {
            "configId": 2,
            "inputHandleId": "variable_node_6_input_2",
            "outputHandleId": "variable_node_6_output_2",
            "triggerConfig": {
              "config": {
                "errorPolicy": {
                  "expired": {
                    "errorLog": {
                      "notify": false
                    },
                    "strategy": "skip"
                  },
                  "nullValue": {
                    "errorLog": {
                      "notify": false
                    },
                    "replaceValue": 0,
                    "strategy": "valueReplace"
                  },
                  "zeroValue": {
                    "errorLog": {
                      "notify": false
                    },
                    "strategy": "skip"
                  }
                },
                "expireDuration": {
                  "duration": 1,
                  "unit": "hour"
                },
                "fromHandleId": "variable_node_5_output_1",
                "fromNodeId": "variable_node_5",
                "fromNodeName": "变量节点5",
                "fromNodeType": "variableNode",
                "fromVar": "test1",
                "fromVarConfigId": 1,
                "fromVarDisplayName": "布尔测试2",
                "fromVarValueType": "boolean"
              },
              "type": "dataflow"
            },
            "updateOperationValue": null,
            "updateVarValueOperation": "set",
            "varDisplayName": "布尔测试",
            "varName": "test1",
            "varOperation": "update",
            "varType": "custom",
            "varValueType": "boolean"
          }
        ]
        "#;

        let configs: Result<Vec<VariableConfig>, _> = serde_json::from_str(json);

        assert!(configs.is_ok(), "反序列化失败: {:?}", configs.err());
        println!("configs: {:#?}", configs);

        let configs = configs.unwrap();
        assert_eq!(configs.len(), 2);

        // 验证第一个配置是 UpdateConfig，并且触发器为数据流
        assert!(configs[0].is_update(), "第一个配置应该是 UpdateConfig");
        match &configs[0] {
            VariableConfig::Update(update_config) => {
                assert_eq!(update_config.config_id(), 1);
                assert_eq!(update_config.var_name(), "percent_test");
                assert!(update_config.update_operation_value().is_none());
                assert_eq!(
                    update_config.update_var_value_operation().to_string(),
                    "add",
                    "第一个配置 updateVarValueOperation 应为 add"
                );

                match update_config.trigger_config() {
                    TriggerConfig::Dataflow(trigger) => {
                        assert_eq!(trigger.from_node_name, "变量节点5");
                        assert_eq!(trigger.from_var_display_name, "百分比测试4");
                        assert_eq!(trigger.error_policy.len(), 3);

                        use crate::node_infra::variable_node::trigger::dataflow::{DataflowErrorPolicy, DataflowErrorType, ErrorLog};

                        match trigger
                            .error_policy
                            .get(&DataflowErrorType::NullValue)
                            .expect("缺失 nullValue 错误策略")
                        {
                            DataflowErrorPolicy::ValueReplace(policy) => {
                                assert!(matches!(&policy.error_log, ErrorLog::NoNotify));
                            }
                            _ => panic!("nullValue 策略应为 ValueReplace"),
                        }

                        match trigger
                            .error_policy
                            .get(&DataflowErrorType::Expired)
                            .expect("缺失 expired 错误策略")
                        {
                            DataflowErrorPolicy::Skip(policy) => {
                                assert!(matches!(&policy.error_log, ErrorLog::NoNotify));
                            }
                            _ => panic!("expired 策略应为 Skip"),
                        }
                    }
                    _ => panic!("第一个配置应为数据流触发器"),
                }
            }
            _ => panic!("第一个配置类型错误"),
        }

        // 验证第二个配置是 UpdateConfig，并且触发器为数据流
        assert!(configs[1].is_update(), "第二个配置应该是 UpdateConfig");
        match &configs[1] {
            VariableConfig::Update(update_config) => {
                assert_eq!(update_config.config_id(), 2);
                assert_eq!(update_config.var_name(), "test1");
                assert!(update_config.update_operation_value().is_none());
                assert_eq!(
                    update_config.update_var_value_operation().to_string(),
                    "set",
                    "第二个配置 updateVarValueOperation 应为 set"
                );

                match update_config.trigger_config() {
                    TriggerConfig::Dataflow(trigger) => {
                        assert_eq!(trigger.from_node_name, "变量节点5");
                        assert_eq!(trigger.from_var_display_name, "布尔测试2");
                        assert_eq!(trigger.error_policy.len(), 3);
                    }
                    _ => panic!("第二个配置应为数据流触发器"),
                }
            }
            _ => panic!("第二个配置类型错误"),
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
          "triggerConfig": {
            "type": "timer",
            "config": {
              "mode": "interval",
              "interval": 1,
              "unit": "hour"
            }
          },
          "varOperation": "get",
          "varValue": 100.5
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "GetConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_get());
        match config {
            VariableConfig::Get(get_config) => {
                assert!(
                    matches!(get_config.trigger_config(), TriggerConfig::Timer(_)),
                    "Get 配置应绑定定时器触发器"
                );
            }
            _ => unreachable!(),
        }

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
          "triggerConfig": {
            "type": "dataflow",
            "config": {
              "errorPolicy": {
                "nullValue": {
                  "strategy": "valueReplace",
                  "replaceValue": 0,
                  "errorLog": {
                    "notify": false
                  }
                }
              },
              "expireDuration": {
                "duration": 5,
                "unit": "minute"
              },
              "fromHandleId": "source_output",
              "fromNodeId": "source_node",
              "fromNodeName": "来源节点",
              "fromNodeType": "variableNode",
              "fromVar": "test_var",
              "fromVarConfigId": 10,
              "fromVarDisplayName": "测试变量",
              "fromVarValueType": "number"
            }
          },
          "varOperation": "update",
          "updateVarValueOperation": "set",
          "updateOperationValue": 200
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "UpdateConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_update());
        match config {
            VariableConfig::Update(update_config) => {
                assert!(
                    matches!(update_config.trigger_config(), TriggerConfig::Dataflow(_)),
                    "Update 配置应绑定数据流触发器"
                );
            }
            _ => unreachable!(),
        }

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
          "triggerConfig": {
            "type": "timer",
            "config": {
              "mode": "interval",
              "interval": 10,
              "unit": "minute"
            }
          },
          "varOperation": "reset",
          "varInitialValue": 0.005
        }
        "#;

        let config: Result<VariableConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok(), "ResetConfig 反序列化失败: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_reset());
        match config {
            VariableConfig::Reset(reset_config) => {
                assert!(
                    matches!(reset_config.trigger_config(), TriggerConfig::Timer(_)),
                    "Reset 配置应绑定定时器触发器"
                );
            }
            _ => unreachable!(),
        }

        println!("✅ ResetConfig 反序列化成功！");
    }

    // #[test]
    // fn test_deserialize_backtest_config() {
    //     let json = r#"
    //     {
    //       "backtestConfig": {
    //         "dataSource": "exchange",
    //         "exchangeModeConfig": {
    //           "selectedAccount": {
    //             "accountName": "账号1",
    //             "availableBalance": 0,
    //             "exchange": "binance",
    //             "id": 1
    //           }
    //         },
    //         "variableConfigs": [
    //           {
    //             "configId": 1,
    //             "inputHandleId": "variable_node_6_input_1",
    //             "outputHandleId": "variable_node_6_output_1",
    //             "triggerConfig": {
    //               "config": {
    //                 "errorPolicy": {
    //                   "expired": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "strategy": "skip"
    //                   },
    //                   "nullValue": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "replaceValue": -12312313,
    //                     "strategy": "valueReplace"
    //                   },
    //                   "zeroValue": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "strategy": "skip"
    //                   }
    //                 },
    //                 "expireDuration": {
    //                   "duration": 1,
    //                   "unit": "hour"
    //                 },
    //                 "fromHandleId": "variable_node_5_output_4",
    //                 "fromNodeId": "variable_node_5",
    //                 "fromNodeName": "变量节点5",
    //                 "fromNodeType": "variableNode",
    //                 "fromVar": "percent_test",
    //                 "fromVarConfigId": 4,
    //                 "fromVarDisplayName": "百分比测试4",
    //                 "fromVarValueType": "percentage"
    //               },
    //               "type": "dataflow"
    //             },
    //             "updateOperationValue": null,
    //             "updateVarValueOperation": "add",
    //             "varDisplayName": "百分比测试",
    //             "varName": "percent_test",
    //             "varOperation": "update",
    //             "varType": "custom",
    //             "varValueType": "percentage"
    //           },
    //           {
    //             "configId": 2,
    //             "inputHandleId": "variable_node_6_input_2",
    //             "outputHandleId": "variable_node_6_output_2",
    //             "triggerConfig": {
    //               "config": {
    //                 "errorPolicy": {
    //                   "expired": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "strategy": "skip"
    //                   },
    //                   "nullValue": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "replaceValue": 0,
    //                     "strategy": "valueReplace"
    //                   },
    //                   "zeroValue": {
    //                     "errorLog": {
    //                       "notify": false
    //                     },
    //                     "strategy": "skip"
    //                   }
    //                 },
    //                 "expireDuration": {
    //                   "duration": 1,
    //                   "unit": "hour"
    //                 },
    //                 "fromHandleId": "variable_node_5_output_1",
    //                 "fromNodeId": "variable_node_5",
    //                 "fromNodeName": "变量节点5",
    //                 "fromNodeType": "variableNode",
    //                 "fromVar": "test1",
    //                 "fromVarConfigId": 1,
    //                 "fromVarDisplayName": "布尔测试2",
    //                 "fromVarValueType": "boolean"
    //               },
    //               "type": "dataflow"
    //             },
    //             "updateOperationValue": null,
    //             "updateVarValueOperation": "set",
    //             "varDisplayName": "布尔测试",
    //             "varName": "test1",
    //             "varOperation": "update",
    //             "varType": "custom",
    //             "varValueType": "boolean"
    //           }
    //         ]
    //       },
    //       "nodeName": "变量节点6",
    //       "strategyId": 2
    //     }
    //     "#;

    //     // use crate::node_infra::variable_node::variable_node_backtest_config::VariableNodeBacktestConfig;

    //     #[derive(serde::Deserialize)]
    //     #[serde(rename_all = "camelCase")]
    //     struct TestNodeData {
    //         strategy_id: i32,
    //         node_name: String,
    //         backtest_config: Option<VariableNodeBacktestConfig>,
    //     }

    //     let node_data: Result<TestNodeData, _> = serde_json::from_str(json);
    //     assert!(node_data.is_ok(), "完整配置反序列化失败: {:?}", node_data.err());

    //     let node_data = node_data.unwrap();
    //     assert_eq!(node_data.strategy_id, 2);
    //     assert_eq!(node_data.node_name, "变量节点6");
    //     assert!(node_data.backtest_config.is_some());

    //     let backtest_config = node_data.backtest_config.unwrap();
    //     assert_eq!(backtest_config.variable_configs.len(), 2);

    //     let exchange_mode = backtest_config
    //         .exchange_mode_config
    //         .as_ref()
    //         .expect("exchangeModeConfig 应存在并包含 selectedAccount");
    //     assert_eq!(exchange_mode.selected_account.account_name, "账号1");

    //     assert!(
    //         backtest_config.variable_configs.iter().all(|config| config.is_update()),
    //         "所有回测变量配置都应为 Update 操作"
    //     );

    //     println!("✅ 完整的回测配置反序列化成功！");
    //     println!("   - 配置数量: {}", backtest_config.variable_configs.len());
    //     println!(
    //         "   - 所有配置使用数据流触发器: {}",
    //         backtest_config
    //             .variable_configs
    //             .iter()
    //             .all(|config| matches!(config.trigger_config(), TriggerConfig::Dataflow(_)))
    //     );
    // }
}
