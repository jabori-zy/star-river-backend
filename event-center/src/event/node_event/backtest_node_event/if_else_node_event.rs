use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;
use strum::Display;

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
    pub case_id: Option<i32>,
}

impl ConditionMatchPayload {
    pub fn new(play_index: PlayIndex, case_id: Option<i32>) -> Self {
        Self { play_index, case_id }
    }
}
