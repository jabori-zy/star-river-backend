use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use strum::EnumString;
use strum_macros::Display;



#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum ComparisonOperator {
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
    #[serde(rename = "varType")]
    pub var_type: VarType,
    #[serde(rename = "varibale")]  // 注意：保持与 JSON 中的拼写一致
    pub variable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(rename = "comparisonOperator")]
    pub comparison_operator: ComparisonOperator,
    #[serde(rename = "leftVariable")]
    pub left_variable: Variable,
    #[serde(rename = "rightVariable")]
    pub right_variable: Variable,
}

// 逻辑操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOperator {
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
    pub conditions: Vec<Condition>,
    #[serde(rename = "logicalOperator")]
    pub logic_operator: LogicOperator,
}

// 使用示例
impl Case {
    pub fn from_json(json_str: &str) -> Result<Case, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}