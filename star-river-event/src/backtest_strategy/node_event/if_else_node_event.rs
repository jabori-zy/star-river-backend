use derive_more::From;
use serde::{Deserialize, Serialize};
use strategy_core::event::node::NodeEvent;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum IfElseNodeEvent {
    CaseTrue(CaseTrueEvent),
    CaseFalse(CaseFalseEvent),
    ElseTrue(ElseTrueEvent),
}

pub type CaseTrueEvent = NodeEvent<CaseTruePayload>;
pub type CaseFalseEvent = NodeEvent<CaseFalsePayload>;
pub type ElseTrueEvent = NodeEvent<ElseTruePayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseTruePayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
    pub case_id: i32,
}

impl CaseTruePayload {
    pub fn new(play_index: i32, case_id: i32) -> Self {
        Self { play_index, case_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFalsePayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
    pub case_id: i32,
}

impl CaseFalsePayload {
    pub fn new(play_index: i32, case_id: i32) -> Self {
        Self { play_index, case_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseTruePayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
}

impl ElseTruePayload {
    pub fn new(play_index: i32) -> Self {
        Self { play_index }
    }
}
