use super::super::{BacktestNodeEvent, NodeEvent};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::position::virtual_position::VirtualPosition;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum PositionManagementNodeEvent {
    #[strum(serialize = "position-created")]
    #[serde(rename = "position-created")]
    PositionCreated(PositionCreatedEvent),

    #[strum(serialize = "position-updated")]
    #[serde(rename = "position-updated")]
    PositionUpdated(PositionUpdatedEvent),

    #[strum(serialize = "position-closed")]
    #[serde(rename = "position-closed")]
    PositionClosed(PositionClosedEvent),
}

// 类型别名
pub type PositionCreatedEvent = NodeEvent<PositionCreatedPayload>;
pub type PositionUpdatedEvent = NodeEvent<PositionUpdatedPayload>;
pub type PositionClosedEvent = NodeEvent<PositionClosedPayload>;

// 载荷类型定义
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
