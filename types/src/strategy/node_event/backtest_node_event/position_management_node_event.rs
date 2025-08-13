use serde::{Deserialize, Serialize};
use strum::Display;
use crate::strategy::node_event::BacktestNodeEvent;
use crate::position::virtual_position::VirtualPosition;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
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


impl From<PositionManagementNodeEvent> for BacktestNodeEvent {
    fn from(event: PositionManagementNodeEvent) -> Self {
        BacktestNodeEvent::PositionManagementNode(event)
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,


    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionClosedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "virtualPosition")]
    pub virtual_position: VirtualPosition,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}