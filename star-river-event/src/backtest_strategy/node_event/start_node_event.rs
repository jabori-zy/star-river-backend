use strategy_core::event::node::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum StartNodeEvent {
    // KlinePlayFinished(KlinePlayFinishedEvent), // K-line play finished
    KlinePlay(KlinePlayEvent), // K-line tick (signal count: use this value to request cache index)
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
