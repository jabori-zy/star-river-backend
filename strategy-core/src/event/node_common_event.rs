use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};

use crate::event::{log_event::NodeStateLogEvent, node::NodeEvent, strategy_event::StrategyRunningLogEvent};

#[derive(Debug, Clone, Serialize, From)]
#[serde(tag = "event_type")]
pub enum CommonEvent {
    Trigger(TriggerEvent),               // Trigger event
    ExecuteOver(ExecuteOverEvent),       // Execute over
    RunningLog(StrategyRunningLogEvent), // Running log
    StateLog(NodeStateLogEvent),         // State log
}

impl CommonEvent {
    pub fn node_id(&self) -> &NodeId {
        match self {
            CommonEvent::Trigger(event) => event.node_id(),
            CommonEvent::ExecuteOver(event) => event.node_id(),
            CommonEvent::RunningLog(event) => event.node_id(),
            CommonEvent::StateLog(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            CommonEvent::Trigger(event) => event.node_name(),
            CommonEvent::ExecuteOver(event) => event.node_name(),
            CommonEvent::RunningLog(event) => event.node_name(),
            CommonEvent::StateLog(event) => event.node_name(),
        }
    }

    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            CommonEvent::Trigger(event) => event.output_handle_id(),
            CommonEvent::ExecuteOver(event) => event.output_handle_id(),
            CommonEvent::RunningLog(event) => &event.output_handle_id(),
            CommonEvent::StateLog(event) => &event.output_handle_id(),
        }
    }
}

// Type aliases

pub type TriggerEvent = NodeEvent<TriggerPayload>;
pub type ExecuteOverEvent = NodeEvent<ExecuteOverPayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPayload {
    #[serde(rename = "cycleId")]
    pub cycle_id: CycleId,
}

impl TriggerPayload {
    pub fn new(cycle_id: CycleId) -> Self {
        Self { cycle_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverPayload {
    #[serde(rename = "cycleId")]
    pub cycle_id: CycleId,

    #[serde(rename = "configId")]
    pub config_id: Option<i32>,
}

impl ExecuteOverPayload {
    pub fn new(cycle_id: CycleId, config_id: Option<i32>) -> Self {
        Self { cycle_id, config_id }
    }
}
