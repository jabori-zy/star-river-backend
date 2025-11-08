use crate::event::node::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::event::log_event::{StrategyRunningLogEvent, NodeStateLogEvent};
use star_river_core::custom_type::CycleId;



#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum CommonEvent {
    Trigger(TriggerEvent), // Trigger event
    ExecuteOver(ExecuteOverEvent),       // Execute over
    RunningLog(StrategyRunningLogEvent), // Running log
    StateLog(NodeStateLogEvent), // State log
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
}

impl ExecuteOverPayload {
    pub fn new(cycle_id: CycleId) -> Self {
        Self { cycle_id }
    }
}