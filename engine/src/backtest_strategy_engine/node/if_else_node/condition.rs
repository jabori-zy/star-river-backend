use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::EnumString;
use strum_macros::Display;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum ComparisonSymbol {
    #[serde(rename = ">")]
    #[strum(serialize = ">")]
    GreaterThan,

    #[serde(rename = "<")]
    #[strum(serialize = "<")]
    LessThan,

    #[serde(rename = "=")]
    #[strum(serialize = "=")]
    Equal,

    #[serde(rename = "!=")]
    #[strum(serialize = "!=")]
    NotEqual,

    #[serde(rename = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,

    #[serde(rename = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarType {
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "constant")]
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    #[serde(rename = "nodeId")]
    pub node_id: Option<String>,

    #[serde(rename = "nodeName")]
    pub node_name: Option<String>,

    #[serde(rename = "varType")]
    pub var_type: VarType,

    #[serde(rename = "outputHandleId")]
    pub output_handle_id: Option<String>,

    #[serde(rename = "variableConfigId")]
    pub variable_config_id: Option<i32>,

    #[serde(rename = "variable")] // 注意：保持与 JSON 中的拼写一致
    pub variable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "conditionId")]
    pub condition_id: i32,
    #[serde(rename = "comparisonSymbol")]
    pub comparison_symbol: ComparisonSymbol,
    #[serde(rename = "leftVariable")]
    pub left_variable: Variable,
    #[serde(rename = "rightVariable")]
    pub right_variable: Variable,
}

// 条件结果
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionResult {
    #[serde(rename = "conditionId")]
    pub condition_id: i32,
    #[serde(rename = "leftVariable")]
    pub left_variable: Variable,
    #[serde(rename = "rightVariable")]
    pub right_variable: Variable,
    #[serde(rename = "comparisonSymbol")]
    pub comparison_symbol: ComparisonSymbol,
    #[serde(rename = "leftValue")]
    pub left_value: Option<f64>,
    #[serde(rename = "rightValue")]
    pub right_value: Option<f64>,
    #[serde(rename = "conditionResult")]
    pub condition_result: bool,
}

impl serde::Serialize for ConditionResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let left_value_str = self.left_value.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());

        let right_value_str = self.right_value.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());

        let right_variable_name = match self.right_variable.var_type {
            VarType::Variable => self.right_variable.variable.clone(),
            VarType::Constant => "constant".to_string(),
        };

        let condition_str = format!(
            "condition{}: {}: {} {} {}: {}",
            self.condition_id, self.left_variable.variable, left_value_str, self.comparison_symbol, right_variable_name, right_value_str
        );

        serializer.serialize_str(&condition_str)
    }
}

// 逻辑操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalSymbol {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

// 分支
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    #[serde(rename = "caseId")]
    pub case_id: i32,
    #[serde(rename = "outputHandleId")]
    pub output_handle_id: String,

    #[serde(rename = "conditions")]
    pub conditions: Vec<Condition>,

    #[serde(rename = "logicalSymbol")]
    pub logical_symbol: LogicalSymbol,
}

// 使用示例
impl Case {
    pub fn from_json(json_str: &str) -> Result<Case, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
