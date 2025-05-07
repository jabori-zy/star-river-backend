
pub mod chart_message;

use types::strategy::node_message::NodeMessage;
use serde::{Serialize, Deserialize};
use strum::Display;
use crate::strategy_event::chart_message::ChartMessage;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum StrategyEvent {
    #[strum(serialize = "node-message")]
    #[serde(rename = "node_message")]
    NodeMessage(NodeMessage), // 节点消息
    #[strum(serialize = "chart-message")]
    #[serde(rename = "chart_message")]
    ChartMessage(ChartMessage), // 图表消息
}





