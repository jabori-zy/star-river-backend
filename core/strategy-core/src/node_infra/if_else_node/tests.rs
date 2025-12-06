#[cfg(test)]
mod tests {
    use crate::node_infra::if_else_node::*;
    use rust_decimal::Decimal;
    use serde_json;

    #[test]
    fn test_deserialize_if_else_node_backtest_config() {
        let json = r#"
        {
          "backtestConfig": {
              "cases": [
                {
                  "caseId": 1,
                  "conditions": [
                    {
                      "comparisonSymbol": "is",
                      "conditionId": 1,
                      "left": {
                        "nodeId": "variable_node_3",
                        "nodeName": "变量节点3",
                        "nodeType": "variableNode",
                        "outputHandleId": "variable_node_3_output_3",
                        "varConfigId": 3,
                        "varDisplayName": "是否开盘3",
                        "varName": "is_market_open",
                        "varType": "variable",
                        "varValueType": "boolean"
                      },
                      "right": {
                        "varType": "constant",
                        "varValue": true,
                        "varValueType": "boolean"
                      }
                    },
                    {
                      "comparisonSymbol": "=",
                      "conditionId": 2,
                      "left": {
                        "nodeId": "variable_node_3",
                        "nodeName": "变量节点3",
                        "nodeType": "variableNode",
                        "outputHandleId": "variable_node_3_output_2",
                        "varConfigId": 2,
                        "varDisplayName": "当前时间2",
                        "varName": "current_time",
                        "varType": "variable",
                        "varValueType": "time"
                      },
                      "right": {
                        "varType": "constant",
                        "varValue": "2025-10-19 20:02:00 +08:00",
                        "varValueType": "time"
                      }
                    },
                    {
                      "comparisonSymbol": "=",
                      "conditionId": 3,
                      "left": {
                        "nodeId": "variable_node_3",
                        "nodeName": "变量节点3",
                        "nodeType": "variableNode",
                        "outputHandleId": "variable_node_3_output_4",
                        "varConfigId": 4,
                        "varDisplayName": "累计收益率4",
                        "varName": "cumulative_yield",
                        "varType": "variable",
                        "varValueType": "percentage"
                      },
                      "right": {
                        "varType": "constant",
                        "varValue": 123123123,
                        "varValueType": "percentage"
                      }
                    }
                  ],
                  "id": 1,
                  "logicalSymbol": "and",
                  "outputHandleId": "if_else_node_4_output_1"
                }
              ]
            }
        }
        "#;

        #[derive(serde::Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct TestNodeData {
            backtest_config: IfElseNodeBacktestConfig,
        }

        let result: Result<TestNodeData, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Deserialization failed: {:?}", result.err());

        println!("✅ IfElseNode backtest config deserialized successfully! {:#?}", result.unwrap().backtest_config);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use crate::variable::custom_variable::{VariableValue, VariableValueType};

        // Create a complete configuration
        let config = IfElseNodeBacktestConfig {
            cases: vec![Case {
                case_id: 1,
                output_handle_id: "output_1".to_string(),
                logical_symbol: LogicalSymbol::And,
                conditions: vec![Condition {
                    condition_id: 1,
                    left: Variable {
                        node_id: "node_1".to_string(),
                        node_name: "Node 1".to_string(),
                        node_type: "variableNode".to_string(),
                        output_handle_id: "output_1".to_string(),
                        var_config_id: 1,
                        var_value_type: VariableValueType::Number,
                        var_display_name: "Variable 1".to_string(),
                        var_name: "var1".to_string(),
                    },
                    comparison_symbol: ComparisonSymbol::GreaterThan,
                    right: FormulaRight::Constant(Constant {
                        // var_value_type: VariableValueType::Number,
                        var_value: VariableValue::Number(Decimal::from(100)),
                    }),
                }],
            }],
        };

        // Serialize
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Deserialize
        let result: Result<IfElseNodeBacktestConfig, _> = serde_json::from_str(&json);
        assert!(result.is_ok(), "Round-trip test failed: {:?}", result.err());

        println!("✅ Serialization and deserialization round-trip test successful!");
    }
}
