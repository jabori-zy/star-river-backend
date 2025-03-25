use types::strategy::message::NodeMessage;
use serde::{Serialize, Deserialize};
use strum::Display;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum StrategyEvent {
    #[strum(serialize = "node-message")]
    #[serde(rename = "node_message")]
    NodeMessage(NodeMessage),
}





