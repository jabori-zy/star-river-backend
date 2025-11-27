use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use super::custom_variable::VariableValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, ToSchema)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SysVariableType {
    CurrentTime,

    IsMarketOpen,

    IsMarketClosed,

    IsTradable,

    TotalPositionNumber,

    PositionNumber, //持仓数量

    TotalFilledOrderNumber, // 总成交订单数量

    FilledOrderNumber, // 已成交订单数量
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SysVariable {
    pub var_name: SysVariableType, // 变量名称
    pub var_display_name: String,  // 变量显示名称
    pub symbol: Option<String>,    // 交易对
    pub var_value: VariableValue,  // 变量值
}

impl serde::Serialize for SysVariable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("SysVariable", 5)?;
        state.serialize_field("varType", "system")?;
        state.serialize_field("varName", &self.var_name)?;
        state.serialize_field("varDisplayName", &self.var_display_name)?;
        state.serialize_field("symbol", &self.symbol)?;
        state.serialize_field("varValueType", &self.var_value.value_type())?;
        state.serialize_field("varValue", &self.var_value)?;
        state.end()
    }
}

impl SysVariable {
    pub fn new(var_name: SysVariableType, var_display_name: String, symbol: Option<String>, var_value: VariableValue) -> Self {
        Self {
            var_name,
            var_display_name,
            symbol,
            var_value,
        }
    }
}

// impl From<StrategySysVariableModel> for SysVariable {
//     fn from(model: StrategySysVariableModel) -> Self {
//         Self {
//             id: model.id,
//             strategy_id: model.strategy_id,
//             position_number: model.position_number,
//             create_time: model.create_time,
//             update_time: model.update_time,
//         }
//     }
// }
