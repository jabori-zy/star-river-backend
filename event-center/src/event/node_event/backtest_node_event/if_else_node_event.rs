use super::super::super::strategy_event::StrategyRunningLogEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum IfElseNodeEvent {
    #[strum(serialize = "running-log")]
    #[serde(rename = "running-log")]
    RunningLog(StrategyRunningLogEvent),
}
