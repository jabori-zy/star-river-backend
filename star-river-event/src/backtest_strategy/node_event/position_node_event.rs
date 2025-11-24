use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};
use strategy_core::event::node::NodeEvent;
use strum::Display;
use virtual_trading::types::VirtualPosition;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum PositionNodeEvent {
    #[strum(serialize = "position-created-event")]
    #[serde(rename = "position-created-event")]
    PositionCreated(PositionCreatedEvent),

    #[strum(serialize = "position-updated-event")]
    #[serde(rename = "position-updated-event")]
    PositionUpdated(PositionUpdatedEvent),

    #[strum(serialize = "position-closed-event")]
    #[serde(rename = "position-closed-event")]
    PositionClosed(PositionClosedEvent),
}

impl PositionNodeEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            PositionNodeEvent::PositionCreated(event) => event.cycle_id(),
            PositionNodeEvent::PositionUpdated(event) => event.cycle_id(),
            PositionNodeEvent::PositionClosed(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            PositionNodeEvent::PositionCreated(event) => event.datetime(),
            PositionNodeEvent::PositionUpdated(event) => event.datetime(),
            PositionNodeEvent::PositionClosed(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            PositionNodeEvent::PositionCreated(event) => event.node_id(),
            PositionNodeEvent::PositionUpdated(event) => event.node_id(),
            PositionNodeEvent::PositionClosed(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            PositionNodeEvent::PositionCreated(event) => event.node_name(),
            PositionNodeEvent::PositionUpdated(event) => event.node_name(),
            PositionNodeEvent::PositionClosed(event) => event.node_name(),
        }
    }

    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            PositionNodeEvent::PositionCreated(event) => event.output_handle_id(),
            PositionNodeEvent::PositionUpdated(event) => event.output_handle_id(),
            PositionNodeEvent::PositionClosed(event) => event.output_handle_id(),
        }
    }
}

// Type aliases
pub type PositionCreatedEvent = NodeEvent<PositionCreatedPayload>;
pub type PositionUpdatedEvent = NodeEvent<PositionUpdatedPayload>;
pub type PositionClosedEvent = NodeEvent<PositionClosedPayload>;

// Payload type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionCreatedPayload {
    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,
}

impl PositionCreatedPayload {
    pub fn new(virtual_position: VirtualPosition) -> Self {
        Self { virtual_position }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdatedPayload {
    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,
}

impl PositionUpdatedPayload {
    pub fn new(virtual_position: VirtualPosition) -> Self {
        Self { virtual_position }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionClosedPayload {
    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,
}

impl PositionClosedPayload {
    pub fn new(virtual_position: VirtualPosition) -> Self {
        Self { virtual_position }
    }
}
