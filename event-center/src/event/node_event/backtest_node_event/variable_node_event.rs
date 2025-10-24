use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::node::variable_node::variable_operation::UpdateVarValueOperation;
use star_river_core::strategy::custom_variable::{CustomVariable, VariableValue};
use star_river_core::strategy::sys_varibale::{SysVariable, SysVariableType};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum VariableNodeEvent {
    #[strum(serialize = "sys-variable-update-event")]
    #[serde(rename = "sys-variable-update-event")]
    SysVariableUpdate(SysVariableUpdateEvent), // 系统变量更新


    #[strum(serialize = "custom-variable-update-event")]
    #[serde(rename = "custom-variable-update-event")]
    CustomVariableUpdate(CustomVariableUpdateEvent),
}

// 类型别名
pub type SysVariableUpdateEvent = NodeEvent<SysVariableUpdatePayload>;
pub type CustomVariableUpdateEvent = NodeEvent<CustomVariableUpdatePayload>;

// 载荷类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysVariableUpdatePayload {
    pub play_index: PlayIndex,
    pub variable_config_id: i32, // 变量配置id
    pub sys_variable: SysVariable
}

impl SysVariableUpdatePayload {
    pub fn new(
        play_index: PlayIndex,
        variable_config_id: i32, 
        sys_variable: SysVariable,
    ) -> Self {
        Self {
            play_index,
            variable_config_id,
            sys_variable,
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomVariableUpdatePayload {
    pub play_index: PlayIndex,
    pub variable_config_id: i32,
    pub var_operation: String, // get, update, reset
    pub update_operation: Option<UpdateVarValueOperation>, // 更新操作，如果为空，则表示获取变量值
    pub update_operation_value: Option<VariableValue>, // 更新操作值，如果为空，则表示获取变量值
    pub custom_variable: CustomVariable,

}


impl CustomVariableUpdatePayload {
    pub fn new(
        play_index: PlayIndex, 
        variable_config_id: i32, 
        var_op: String,
        update_operation: Option<UpdateVarValueOperation>,
        update_operation_value: Option<VariableValue>,
        custom_variable: CustomVariable,
    ) -> Self {
        Self { 
            play_index, 
            variable_config_id, 
            var_operation: var_op,
            update_operation,
            update_operation_value,
            custom_variable,
        }
    }
}