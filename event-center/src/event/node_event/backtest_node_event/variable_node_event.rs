use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::strategy::custom_variable::VariableValue;
use star_river_core::strategy::sys_varibale::SysVariable;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum VariableNodeEvent {
    #[strum(serialize = "sys-variable-updated-event")]
    #[serde(rename = "sys-variable-updated-event")]
    SysVariableUpdated(SysVariableUpdatedEvent), // 系统变量更新


    #[strum(serialize = "custom-variable-update-event")]
    #[serde(rename = "custom-variable-update-event")]
    CustomVariableUpdate(CustomVariableUpdateEvent),
}

// 类型别名
pub type SysVariableUpdatedEvent = NodeEvent<SysVariableUpdatedPayload>;
pub type CustomVariableUpdateEvent = NodeEvent<CustomVariableUpdatePayload>;

// 载荷类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysVariableUpdatedPayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,

    #[serde(rename = "variableConfigId")]
    pub variable_config_id: i32, // 变量配置id

    #[serde(rename = "variable")]
    pub variable: SysVariable,

    #[serde(rename = "variableValue")]
    pub variable_value: f64,
}

impl SysVariableUpdatedPayload {
    pub fn new(play_index: i32, variable_config_id: i32, variable: SysVariable, variable_value: f64) -> Self {
        Self {
            play_index,
            variable_config_id,
            variable,
            variable_value,
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomVariableUpdatePayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,

    #[serde(rename = "nodeId")]
    pub node_id: NodeId,

    #[serde(rename = "variableConfigId")]
    pub variable_config_id: i32,

    #[serde(rename = "varName")]
    pub var_name: String,

    #[serde(rename = "varValue")]
    pub var_value: VariableValue,
}


impl CustomVariableUpdatePayload {
    pub fn new(play_index: PlayIndex, node_id: NodeId, variable_config_id: i32, var_name: String, var_value: VariableValue) -> Self {
        Self { play_index, node_id, variable_config_id, var_name, var_value }
    }
}