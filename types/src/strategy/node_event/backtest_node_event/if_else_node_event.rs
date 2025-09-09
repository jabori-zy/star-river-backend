use super::super::BacktestNodeEvent;
use super::super::StrategyRunningLogEvent;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum IfElseNodeEvent {
    #[strum(serialize = "running-log")]
    #[serde(rename = "running-log")]
    RunningLog(StrategyRunningLogEvent),
}

impl From<IfElseNodeEvent> for BacktestNodeEvent {
    fn from(event: IfElseNodeEvent) -> Self {
        BacktestNodeEvent::IfElseNode(event)
    }
}
