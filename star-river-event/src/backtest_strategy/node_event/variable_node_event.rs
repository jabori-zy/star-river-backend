use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};
use strategy_core::{
    event::node::NodeEvent,
    node_infra::variable_node::variable_operation::UpdateVarValueOperation,
    variable::{
        custom_variable::{CustomVariable, VariableValue},
        sys_varibale::SysVariable,
    },
};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum VariableNodeEvent {
    #[strum(serialize = "sys-variable-update-event")]
    #[serde(rename = "sys-variable-update-event")]
    SysVarUpdate(SysVarUpdateEvent), // System variable update

    #[strum(serialize = "custom-variable-update-event")]
    #[serde(rename = "custom-variable-update-event")]
    CustomVarUpdate(CustomVarUpdateEvent),
}

impl VariableNodeEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            VariableNodeEvent::SysVarUpdate(event) => event.cycle_id(),
            VariableNodeEvent::CustomVarUpdate(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            VariableNodeEvent::SysVarUpdate(event) => event.datetime(),
            VariableNodeEvent::CustomVarUpdate(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            VariableNodeEvent::SysVarUpdate(event) => event.node_id(),
            VariableNodeEvent::CustomVarUpdate(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            VariableNodeEvent::SysVarUpdate(event) => event.node_name(),
            VariableNodeEvent::CustomVarUpdate(event) => event.node_name(),
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            VariableNodeEvent::SysVarUpdate(event) => event.output_handle_id(),
            VariableNodeEvent::CustomVarUpdate(event) => event.output_handle_id(),
        }
    }
}

// Type aliases
pub type SysVarUpdateEvent = NodeEvent<SysVarUpdatePayload>;
pub type CustomVarUpdateEvent = NodeEvent<CustomVarUpdatePayload>;

// Payload type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysVarUpdatePayload {
    pub cycle_id: CycleId,
    pub variable_config_id: i32, // Variable config ID
    pub sys_variable: SysVariable,
}

impl SysVarUpdatePayload {
    pub fn new(cycle_id: CycleId, variable_config_id: i32, sys_variable: SysVariable) -> Self {
        Self {
            cycle_id,
            variable_config_id,
            sys_variable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomVarUpdatePayload {
    pub cycle_id: CycleId,
    pub variable_config_id: i32,
    pub var_operation: String,                             // get, update, reset
    pub update_operation: Option<UpdateVarValueOperation>, // Update operation, if empty, it means getting variable value
    pub update_operation_value: Option<VariableValue>,     // Update operation value, if empty, it means getting variable value
    pub custom_variable: CustomVariable,
}

impl CustomVarUpdatePayload {
    pub fn new(
        cycle_id: CycleId,
        variable_config_id: i32,
        var_op: String,
        update_operation: Option<UpdateVarValueOperation>,
        update_operation_value: Option<VariableValue>,
        custom_variable: CustomVariable,
    ) -> Self {
        Self {
            cycle_id,
            variable_config_id,
            var_operation: var_op,
            update_operation,
            update_operation_value,
            custom_variable,
        }
    }
}
