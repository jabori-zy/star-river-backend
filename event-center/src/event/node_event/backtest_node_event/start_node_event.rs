use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum StartNodeEvent {
    // KlinePlayFinished(KlinePlayFinishedEvent), // k线播放完毕
    KlinePlay(KlinePlayEvent),                 // K线跳动(信号计数:根据这个值去请求缓存的下标)
}

// pub type KlinePlayFinishedEvent = NodeEvent<KlinePlayFinishedPayload>;
pub type KlinePlayEvent = NodeEvent<KlinePlayPayload>;


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KlinePlayFinishedPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: PlayIndex,
// }

// impl KlinePlayFinishedPayload {
//     pub fn new(play_index: PlayIndex) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayPayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,
}

impl KlinePlayPayload {
    pub fn new(play_index: PlayIndex) -> Self {
        Self { play_index }
    }
}