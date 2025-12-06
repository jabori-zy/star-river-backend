use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};
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
    pub fn cycle_id(&self) -> CycleId {
        match self {
            IfElseNodeEvent::CaseTrue(event) => event.cycle_id(),
            IfElseNodeEvent::CaseFalse(event) => event.cycle_id(),
            IfElseNodeEvent::ElseTrue(event) => event.cycle_id(),
            IfElseNodeEvent::ElseFalse(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            IfElseNodeEvent::CaseTrue(event) => event.datetime(),
            IfElseNodeEvent::CaseFalse(event) => event.datetime(),
            IfElseNodeEvent::ElseTrue(event) => event.datetime(),
            IfElseNodeEvent::ElseFalse(event) => event.datetime(),
        }
    }
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
    pub case_id: i32,
}

impl CaseTruePayload {
    pub fn new(case_id: i32) -> Self {
        Self { case_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFalsePayload {
    pub case_id: i32,
}

impl CaseFalsePayload {
    pub fn new(case_id: i32) -> Self {
        Self { case_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseTruePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseFalsePayload;
