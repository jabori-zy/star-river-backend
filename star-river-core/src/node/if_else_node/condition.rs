use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::EnumString;
use strum_macros::Display;
use crate::strategy::custom_variable::{VariableValue, VariableValueType};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum ComparisonSymbol {
    #[serde(rename = "=")]
    #[strum(serialize = "=")]
    Equal,

    #[serde(rename = "!=")]
    #[strum(serialize = "!=")]
    NotEqual,

    #[serde(rename = ">")]
    #[strum(serialize = ">")]
    GreaterThan,

    #[serde(rename = "<")]
    #[strum(serialize = "<")]
    LessThan,

    #[serde(rename = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,

    #[serde(rename = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqual,

    #[serde(rename = "is")]
    #[strum(serialize = "is")]
    Is,

    #[serde(rename = "is not")]
    #[strum(serialize = "is not")]
    IsNot,

    #[serde(rename = "contains")]
    #[strum(serialize = "contains")]
    Contains,

    #[serde(rename = "not contains")]
    #[strum(serialize = "not contains")]
    NotContains,

    #[serde(rename = "is in")]
    #[strum(serialize = "is in")]
    IsIn,

    #[serde(rename = "is not in")]
    #[strum(serialize = "is not in")]
    IsNotIn,

    #[serde(rename = "is empty")]
    #[strum(serialize = "is empty")]
    IsEmpty,

    #[serde(rename = "is not empty")]
    #[strum(serialize = "is not empty")]
    IsNotEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarType {
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "constant")]
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "varType", rename_all = "lowercase")]
pub enum FormulaRight {
    Variable(Variable),
    Constant(Constant),
}

impl FormulaRight {
    pub fn is_variable(&self) -> bool {
        matches!(self, FormulaRight::Variable(_))
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, FormulaRight::Constant(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub output_handle_id: String,
    pub var_config_id: i32,
    pub var_value_type: VariableValueType,
    pub var_display_name: String,
    pub var_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Constant {
    // pub var_value_type: VariableValueType,
    pub var_value: VariableValue,
}

impl<'de> serde::Deserialize<'de> for Constant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ConstantHelper {
            var_value_type: VariableValueType,
            var_value: serde_json::Value,
        }

        let helper = ConstantHelper::deserialize(deserializer)?;

        // 使用 VariableValue::from_json_with_type 根据类型解析值
        let var_value = VariableValue::from_json_with_type(helper.var_value, &helper.var_value_type)
            .map_err(D::Error::custom)?;

        Ok(Constant {
            // var_value_type: helper.var_value_type,
            var_value,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub condition_id: i32,
    pub comparison_symbol: ComparisonSymbol,
    pub left: Variable,
    pub right: FormulaRight,
}

// 条件结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionResult {
    pub condition_id: i32,
    pub left: Variable,
    pub right: FormulaRight,
    pub comparison_symbol: ComparisonSymbol,
    pub left_value: VariableValue,
    pub right_value: VariableValue,
    pub condition_result: bool,
}

impl ConditionResult {
    pub fn new(condition: &Condition, left_value: VariableValue, right_value: VariableValue, condition_result: bool) -> Self {
        Self {
            condition_id: condition.condition_id,
            left: condition.left.clone(),
            right: condition.right.clone(),
            comparison_symbol: condition.comparison_symbol.clone(),
            left_value: left_value,
            right_value: right_value,
            condition_result: condition_result,
        }
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
