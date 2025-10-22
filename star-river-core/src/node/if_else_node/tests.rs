#[cfg(test)]
mod tests {
    use crate::node::if_else_node::*;
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
        assert!(result.is_ok(), "反序列化失败: {:?}", result.err());

        println!("✅ IfElseNode 回测配置反序列化成功！{:#?}", result.unwrap().backtest_config);
    }

    #[test]
    fn test_serialize_and_deserialize() {
        use crate::strategy::custom_variable::{VariableValue, VariableValueType};

        // 创建一个完整的配置
        let config = IfElseNodeBacktestConfig {
            cases: vec![Case {
                case_id: 1,
                output_handle_id: "output_1".to_string(),
                logical_symbol: LogicalSymbol::And,
                conditions: vec![Condition {
                    condition_id: 1,
                    left: Variable {
                        node_id: "node_1".to_string(),
                        node_name: "节点1".to_string(),
                        node_type: "variableNode".to_string(),
                        output_handle_id: "output_1".to_string(),
                        var_config_id: 1,
                        var_value_type: VariableValueType::Number,
                        var_display_name: "变量1".to_string(),
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

        // 序列化
        let json = serde_json::to_string_pretty(&config).unwrap();

        // 反序列化
        let result: Result<IfElseNodeBacktestConfig, _> = serde_json::from_str(&json);
        assert!(result.is_ok(), "往返测试失败: {:?}", result.err());

        println!("✅ 序列化和反序列化往返测试成功！");
    }
}
