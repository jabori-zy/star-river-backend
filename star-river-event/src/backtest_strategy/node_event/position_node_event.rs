use derive_more::From;
use serde::{Deserialize, Serialize};
use strategy_core::event::node::NodeEvent;
use strum::Display;
use virtual_trading::types::VirtualPosition;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum PositionManagementNodeEvent {
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
