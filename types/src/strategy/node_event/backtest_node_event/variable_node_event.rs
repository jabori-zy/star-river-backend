use crate::strategy::node_event::BacktestNodeEvent;
use crate::strategy::sys_varibale::SysVariable;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_type")]
pub enum VariableNodeEvent {
    #[strum(serialize = "sys-variable-updated")]
    #[serde(rename = "sys-variable-updated")]
    SysVariableUpdated(SysVariableUpdatedEvent), // 系统变量更新
}

impl From<VariableNodeEvent> for BacktestNodeEvent {
    fn from(event: VariableNodeEvent) -> Self {
        BacktestNodeEvent::Variable(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysVariableUpdatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(rename = "variableConfigId")]
    pub variable_config_id: i32, // 变量配置id

    #[serde(rename = "variable")]
    pub variable: SysVariable,

    #[serde(rename = "variableValue")]
    pub variable_value: f64,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}
