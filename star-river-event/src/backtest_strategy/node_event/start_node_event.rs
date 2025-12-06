use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};
use strategy_core::event::node::NodeEvent;

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum StartNodeEvent {
    KlinePlay(KlinePlayEvent), // K-line tick (signal count: use this value to request cache index)
}

impl StartNodeEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            StartNodeEvent::KlinePlay(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            StartNodeEvent::KlinePlay(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            StartNodeEvent::KlinePlay(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            StartNodeEvent::KlinePlay(event) => event.node_name(),
        }
    }

    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            StartNodeEvent::KlinePlay(event) => event.output_handle_id(),
        }
    }
}

pub type KlinePlayEvent = NodeEvent<KlinePlayPayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayPayload;
