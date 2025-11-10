use derive_more::From;
use serde::{Deserialize, Serialize};
use strategy_core::event::node::NodeEvent;
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
    pub play_index: i32,
    pub case_id: Option<i32>,
}

impl ConditionMatchPayload {
    pub fn new(play_index: i32, case_id: Option<i32>) -> Self {
        Self { play_index, case_id }
    }
}
