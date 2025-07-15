use serde::{Deserialize, Serialize};
use strum::Display;
use crate::order::virtual_order::VirtualOrder;
use crate::strategy::node_event::BacktestNodeEvent;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_type")]
pub enum FuturesOrderNodeEvent {
    #[strum(serialize = "futures-order-created")]
    #[serde(rename = "futures-order-created")]
    FuturesOrderCreated(FuturesOrderCreatedEvent),

    #[strum(serialize = "futures-order-canceled")]
    #[serde(rename = "futures-order-canceled")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent),

    #[strum(serialize = "futures-order-filled")]
    #[serde(rename = "futures-order-filled")]
    FuturesOrderFilled(FuturesOrderFilledEvent),
}


impl From<FuturesOrderNodeEvent> for BacktestNodeEvent {
    fn from(event: FuturesOrderNodeEvent) -> Self {
        BacktestNodeEvent::FuturesOrderNode(event)
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,


    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCanceledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderFilledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}