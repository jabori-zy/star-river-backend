use derive_more::From;
use serde::{Deserialize, Serialize};
use strum::Display;
use super::super::NodeEvent;
use star_river_core::custom_type::PlayIndex;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum IfElseNodeEvent {
    ConditionMatch(ConditionMatchEvent),
}

pub type ConditionMatchEvent = NodeEvent<ConditionMatchPayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionMatchPayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,
}

impl ConditionMatchPayload {
    pub fn new(play_index: PlayIndex) -> Self {
        Self { play_index }
    }
}