use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{HandleId, NodeId, NodeName};
use strategy_core::event::node::NodeEvent;

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum StartNodeEvent {
    // KlinePlayFinished(KlinePlayFinishedEvent), // K-line play finished
    KlinePlay(KlinePlayEvent), // K-line tick (signal count: use this value to request cache index)
}

impl StartNodeEvent {
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

// pub type KlinePlayFinishedEvent = NodeEvent<KlinePlayFinishedPayload>;
pub type KlinePlayEvent = NodeEvent<KlinePlayPayload>;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KlinePlayFinishedPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: i32,
// }

// impl KlinePlayFinishedPayload {
//     pub fn new(play_index: i32) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayPayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
}

impl KlinePlayPayload {
    pub fn new(play_index: i32) -> Self {
        Self { play_index }
    }
}
