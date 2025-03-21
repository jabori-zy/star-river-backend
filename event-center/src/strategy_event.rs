use serde::{Deserialize, Serialize};
use strum::Display;
use types::strategy::message::NodeMessage;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum StrategyEvent {
    #[strum(serialize = "node-message")]
    #[serde(rename = "node_message")]
    NodeMessage(NodeMessage),
}





