use strategy_core::event::node::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};

// TODO: Need to define StrategyRunningLogEvent
// use super::super::super::strategy_event::StrategyRunningLogEvent;
use crate::backtest_strategy::strategy_event::log_event::{StrategyRunningLogEvent, NodeStateLogEvent};

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum CommonEvent {
    // ConditionMatch(ConditionMatchEvent),       // Backtest condition match
    Trigger(TriggerEvent), // Trigger event
    // KlinePlayFinished(KlinePlayFinishedEvent), // K-line play finished
    // KlinePlay(KlinePlayEvent),                 // K-line tick (signal count: use this value to request cache index)
    ExecuteOver(ExecuteOverEvent),       // Execute over
    RunningLog(StrategyRunningLogEvent), // Running log
    StateLog(NodeStateLogEvent), // State log
}

// Type aliases
// pub type ConditionMatchEvent = NodeEvent<ConditionMatchPayload>;
pub type TriggerEvent = NodeEvent<TriggerPayload>;
// pub type KlinePlayFinishedEvent = NodeEvent<KlinePlayFinishedPayload>;
// pub type KlinePlayEvent = NodeEvent<KlinePlayPayload>;
pub type ExecuteOverEvent = NodeEvent<ExecuteOverPayload>;

// Payload type definitions for each event (to avoid From trait conflicts)
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ConditionMatchPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: i32,
// }

// impl ConditionMatchPayload {
//     pub fn new(play_index: i32) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
}

impl TriggerPayload {
    pub fn new(play_index: i32) -> Self {
        Self { play_index }
    }
}

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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KlinePlayPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: i32,
// }

// impl KlinePlayPayload {
//     pub fn new(play_index: i32) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverPayload {
    #[serde(rename = "playIndex")]
    pub play_index: i32,
}

impl ExecuteOverPayload {
    pub fn new(play_index: i32) -> Self {
        Self { play_index }
    }
}
