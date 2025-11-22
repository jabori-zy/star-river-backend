use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{HandleId, NodeId, NodeName};
use strategy_core::event::node::NodeEvent;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum IfElseNodeEvent {
    CaseTrue(CaseTrueEvent),
    CaseFalse(CaseFalseEvent),
    ElseTrue(ElseTrueEvent),
    ElseFalse(ElseFalseEvent),
}

impl IfElseNodeEvent {
    pub fn node_id(&self) -> &NodeId {
        match self {
            IfElseNodeEvent::CaseTrue(event) => event.node_id(),
            IfElseNodeEvent::CaseFalse(event) => event.node_id(),
            IfElseNodeEvent::ElseTrue(event) => event.node_id(),
            IfElseNodeEvent::ElseFalse(event) => event.node_id(),
        }
    }
    pub fn node_name(&self) -> &NodeName {
        match self {
            IfElseNodeEvent::CaseTrue(event) => event.node_name(),
            IfElseNodeEvent::CaseFalse(event) => event.node_name(),
            IfElseNodeEvent::ElseTrue(event) => event.node_name(),
            IfElseNodeEvent::ElseFalse(event) => event.node_name(),
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            IfElseNodeEvent::CaseTrue(event) => event.output_handle_id(),
            IfElseNodeEvent::CaseFalse(event) => event.output_handle_id(),
            IfElseNodeEvent::ElseTrue(event) => event.output_handle_id(),
            IfElseNodeEvent::ElseFalse(event) => event.output_handle_id(),
        }
    }
}

pub type CaseTrueEvent = NodeEvent<CaseTruePayload>;
pub type CaseFalseEvent = NodeEvent<CaseFalsePayload>;
pub type ElseTrueEvent = NodeEvent<ElseTruePayload>;
pub type ElseFalseEvent = NodeEvent<ElseFalsePayload>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseFalsePayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
}

impl ElseFalsePayload {
    pub fn new(play_index: i32) -> Self {
        Self { play_index }
    }
}
