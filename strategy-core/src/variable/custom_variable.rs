use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::ToSchema;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum VariableValueType {
    #[strum(serialize = "number")]
    Number,
    #[strum(serialize = "string")]
    String,
    #[strum(serialize = "boolean")]
    Boolean,
    #[strum(serialize = "time")]
    Time,
    #[strum(serialize = "enum")]
    Enum,
    #[strum(serialize = "percentage")]
    Percentage,
    #[strum(serialize = "null")]
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, ToSchema)]
#[serde(untagged)]
pub enum VariableValue {
    // #[schema(value_type = Decimal, example = "100.00")]
    Number(Decimal),
    String(String),
    Boolean(bool),
    Enum(Vec<String>), // 用于 enum 类型的选项列表
    #[schema(value_type = DateTime, example = "2025-10-19 20:02:00 +08:00")]
    Time(chrono::DateTime<Utc>),
    #[schema(value_type = Decimal, example = "100.00")]
    Percentage(Decimal),
    Null,
}

impl VariableValue {
    /// 根据 VariableValueType 从 JSON Value 解析 VariableValue
    pub fn from_json_with_type(value: serde_json::Value, value_type: &VariableValueType) -> Result<Self, String> {
        match value_type {
            VariableValueType::Number => {
                let num = value.as_f64().ok_or_else(|| "expected number for Number type".to_string())?;
                Decimal::try_from(num)
                    .map(VariableValue::Number)
                    .map_err(|e| format!("failed to convert to Decimal: {}", e))
            }
            VariableValueType::String => value
                .as_str()
                .map(|s| VariableValue::String(s.to_string()))
                .ok_or_else(|| "expected string for String type".to_string()),
            VariableValueType::Boolean => value
                .as_bool()
                .map(VariableValue::Boolean)
                .ok_or_else(|| "expected boolean for Boolean type".to_string()),
            VariableValueType::Time => {
                let time_str = value.as_str().ok_or_else(|| "expected string for Time type".to_string())?;

                // 前端返回的格式："2025-10-19 20:02:00 +08:00"，带时区
                use chrono::DateTime;
                DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S %z")
                    .map(|dt| VariableValue::Time(dt.with_timezone(&chrono::Utc)))
                    .map_err(|e| format!("failed to parse time '{}': {}", time_str, e))
            }
            VariableValueType::Enum => serde_json::from_value::<Vec<String>>(value)
                .map(VariableValue::Enum)
                .map_err(|e| format!("expected array for Enum type: {}", e)),
            VariableValueType::Percentage => {
                let num = value.as_f64().ok_or_else(|| "expected number for Percentage type".to_string())?;
                Decimal::try_from(num)
                    .map(VariableValue::Percentage)
                    .map_err(|e| format!("failed to convert to Decimal: {}", e))
            }
            _ => {
                return Err(format!("unsupported variable value type: {}", value_type.to_string()));
            }
        }
    }

    pub fn value_type(&self) -> String {
        match self {
            VariableValue::Number(_) => VariableValueType::Number.to_string(),
            VariableValue::String(_) => VariableValueType::String.to_string(),
            VariableValue::Boolean(_) => VariableValueType::Boolean.to_string(),
            VariableValue::Enum(_) => VariableValueType::Enum.to_string(),
            VariableValue::Time(_) => VariableValueType::Time.to_string(),
            VariableValue::Percentage(_) => VariableValueType::Percentage.to_string(),
            VariableValue::Null => VariableValueType::Null.to_string(),
        }
    }
}

// PartialEq implementation for VariableValue
impl PartialEq for VariableValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VariableValue::Number(a), VariableValue::Number(b)) => a == b,
            (VariableValue::String(a), VariableValue::String(b)) => a == b,
            (VariableValue::Boolean(a), VariableValue::Boolean(b)) => a == b,
            (VariableValue::Enum(a), VariableValue::Enum(b)) => a == b,
            (VariableValue::Time(a), VariableValue::Time(b)) => a == b,
            (VariableValue::Percentage(a), VariableValue::Percentage(b)) => a == b,
            (VariableValue::Null, VariableValue::Null) => true,
            _ => false,
        }
    }
}

// From trait 实现
impl From<Decimal> for VariableValue {
    fn from(value: Decimal) -> Self {
        Self::Number(value)
    }
}

impl From<f64> for VariableValue {
    fn from(value: f64) -> Self {
        Self::Number(Decimal::try_from(value).unwrap_or(Decimal::ZERO))
    }
}

impl From<i64> for VariableValue {
    fn from(value: i64) -> Self {
        Self::Number(Decimal::from(value))
    }
}

impl From<String> for VariableValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for VariableValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for VariableValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<Vec<String>> for VariableValue {
    fn from(value: Vec<String>) -> Self {
        Self::Enum(value)
    }
}

impl From<chrono::DateTime<Utc>> for VariableValue {
    fn from(value: chrono::DateTime<Utc>) -> Self {
        Self::Time(value)
    }
}

#[derive(Debug, Clone, ToSchema)]
pub struct CustomVariable {
    pub var_name: String,             // 变量名称
    pub var_display_name: String,     // 变量显示名称
    pub initial_value: VariableValue, // 初始值
    pub previous_value: VariableValue, // 前一个值
    pub var_value: VariableValue,     // 变量值
}

impl serde::Serialize for CustomVariable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("CustomVariable", 5)?;
        state.serialize_field("varType", "custom")?;
        state.serialize_field("varName", &self.var_name)?;
        state.serialize_field("varDisplayName", &self.var_display_name)?;
        state.serialize_field("varValueType", &self.var_value.value_type())?;
        state.serialize_field("initialValue", &self.initial_value)?;
        state.serialize_field("previousValue", &self.previous_value)?;
        state.serialize_field("varValue", &self.var_value)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for CustomVariable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct CustomVariableHelper {
            var_name: String,
            var_display_name: String,
            var_value_type: VariableValueType,
            initial_value: serde_json::Value,
            previous_value: serde_json::Value,
            var_value: serde_json::Value,
        }

        let helper = CustomVariableHelper::deserialize(deserializer)?;

        // 使用 VariableValue::from_json_with_type 根据类型解析初始值
        let initial_value = VariableValue::from_json_with_type(helper.initial_value, &helper.var_value_type).map_err(D::Error::custom)?;
        let previous_value = VariableValue::from_json_with_type(helper.previous_value, &helper.var_value_type).map_err(D::Error::custom)?;
        // 使用 VariableValue::from_json_with_type 根据类型解析当前值
        let var_value = VariableValue::from_json_with_type(helper.var_value, &helper.var_value_type).map_err(D::Error::custom)?;

        Ok(CustomVariable {
            var_name: helper.var_name,
            var_display_name: helper.var_display_name,
            initial_value,
            previous_value,
            var_value,
        })
    }
}
