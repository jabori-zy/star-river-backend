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

        assert!(configs.is_ok(), "Deserialization failed: {:?}", configs.err());
        println!("configs: {:#?}", configs);

        let configs = configs.unwrap();
        assert_eq!(configs.len(), 2);

        // Verify first config is UpdateConfig with dataflow trigger
        assert!(configs[0].is_update(), "First config should be UpdateConfig");
        match &configs[0] {
            VariableConfig::Update(update_config) => {
                assert_eq!(update_config.config_id(), 1);
                assert_eq!(update_config.var_name(), "percent_test");
                assert!(update_config.update_operation_value().is_none());
                assert_eq!(
                    update_config.update_var_value_operation().to_string(),
                    "add",
                    "First config updateVarValueOperation should be 'add'"
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
                            .expect("Missing nullValue error policy")
                        {
                            DataflowErrorPolicy::ValueReplace(policy) => {
                                assert!(matches!(&policy.error_log, ErrorLog::NoNotify));
                            }
                            _ => panic!("nullValue policy should be ValueReplace"),
                        }

                        match trigger
                            .error_policy
                            .get(&DataflowErrorType::Expired)
                            .expect("Missing expired error policy")
                        {
                            DataflowErrorPolicy::Skip(policy) => {
                                assert!(matches!(&policy.error_log, ErrorLog::NoNotify));
                            }
                            _ => panic!("expired policy should be Skip"),
                        }
                    }
                    _ => panic!("First config should have dataflow trigger"),
                }
            }
            _ => panic!("First config type error"),
        }

        // Verify second config is UpdateConfig with dataflow trigger
        assert!(configs[1].is_update(), "Second config should be UpdateConfig");
        match &configs[1] {
            VariableConfig::Update(update_config) => {
                assert_eq!(update_config.config_id(), 2);
                assert_eq!(update_config.var_name(), "test1");
                assert!(update_config.update_operation_value().is_none());
                assert_eq!(
                    update_config.update_var_value_operation().to_string(),
                    "set",
                    "Second config updateVarValueOperation should be 'set'"
                );

                match update_config.trigger_config() {
                    TriggerConfig::Dataflow(trigger) => {
                        assert_eq!(trigger.from_node_name, "变量节点5");
                        assert_eq!(trigger.from_var_display_name, "布尔测试2");
                        assert_eq!(trigger.error_policy.len(), 3);
                    }
                    _ => panic!("Second config should have dataflow trigger"),
                }
            }
            _ => panic!("Second config type error"),
        }

        println!("✅ All configurations deserialized correctly!");
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
        assert!(config.is_ok(), "GetConfig deserialization failed: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_get());
        match config {
            VariableConfig::Get(get_config) => {
                assert!(
                    matches!(get_config.trigger_config(), TriggerConfig::Timer(_)),
                    "Get config should have timer trigger"
                );
            }
            _ => unreachable!(),
        }

        println!("✅ GetConfig deserialized successfully!");
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
        assert!(config.is_ok(), "UpdateConfig deserialization failed: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_update());
        match config {
            VariableConfig::Update(update_config) => {
                assert!(
                    matches!(update_config.trigger_config(), TriggerConfig::Dataflow(_)),
                    "Update config should have dataflow trigger"
                );
            }
            _ => unreachable!(),
        }

        println!("✅ UpdateConfig deserialized successfully!");
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
        assert!(config.is_ok(), "ResetConfig deserialization failed: {:?}", config.err());

        let config = config.unwrap();
        assert!(config.is_reset());
        match config {
            VariableConfig::Reset(reset_config) => {
                assert!(
                    matches!(reset_config.trigger_config(), TriggerConfig::Timer(_)),
                    "Reset config should have timer trigger"
                );
            }
            _ => unreachable!(),
        }

        println!("✅ ResetConfig deserialized successfully!");
    }

    // #[test]
    // fn test_deserialize_backtest_config() {
    //     let json = r#"
    //     {
    //       "backtestConfig": {
    //         "dataSource": "exchange",
    //         "exchangeModeConfig": {
    //           "selectedAccount": {
    //             "accountName": "Account1",
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
    //                 "fromNodeName": "VariableNode5",
    //                 "fromNodeType": "variableNode",
    //                 "fromVar": "percent_test",
    //                 "fromVarConfigId": 4,
    //                 "fromVarDisplayName": "PercentTest4",
    //                 "fromVarValueType": "percentage"
    //               },
    //               "type": "dataflow"
    //             },
    //             "updateOperationValue": null,
    //             "updateVarValueOperation": "add",
    //             "varDisplayName": "PercentTest",
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
    //                 "fromNodeName": "VariableNode5",
    //                 "fromNodeType": "variableNode",
    //                 "fromVar": "test1",
    //                 "fromVarConfigId": 1,
    //                 "fromVarDisplayName": "BoolTest2",
    //                 "fromVarValueType": "boolean"
    //               },
    //               "type": "dataflow"
    //             },
    //             "updateOperationValue": null,
    //             "updateVarValueOperation": "set",
    //             "varDisplayName": "BoolTest",
    //             "varName": "test1",
    //             "varOperation": "update",
    //             "varType": "custom",
    //             "varValueType": "boolean"
    //           }
    //         ]
    //       },
    //       "nodeName": "VariableNode6",
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
    //     assert!(node_data.is_ok(), "Full config deserialization failed: {:?}", node_data.err());

    //     let node_data = node_data.unwrap();
    //     assert_eq!(node_data.strategy_id, 2);
    //     assert_eq!(node_data.node_name, "VariableNode6");
    //     assert!(node_data.backtest_config.is_some());

    //     let backtest_config = node_data.backtest_config.unwrap();
    //     assert_eq!(backtest_config.variable_configs.len(), 2);

    //     let exchange_mode = backtest_config
    //         .exchange_mode_config
    //         .as_ref()
    //         .expect("exchangeModeConfig should exist and contain selectedAccount");
    //     assert_eq!(exchange_mode.selected_account.account_name, "Account1");

    //     assert!(
    //         backtest_config.variable_configs.iter().all(|config| config.is_update()),
    //         "All backtest variable configs should be Update operation"
    //     );

    //     println!("✅ Full backtest config deserialized successfully!");
    //     println!("   - Config count: {}", backtest_config.variable_configs.len());
    //     println!(
    //         "   - All configs use dataflow trigger: {}",
    //         backtest_config
    //             .variable_configs
    //             .iter()
    //             .all(|config| matches!(config.trigger_config(), TriggerConfig::Dataflow(_)))
    //     );
    // }
}
