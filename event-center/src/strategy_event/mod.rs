
use types::strategy::node_message::NodeMessage;
use serde::{Serialize, Deserialize};
use strum::Display;
use std::collections::HashMap;
use types::cache::{CacheKey, CacheValue};
use std::sync::Arc;
use crate::Event;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum StrategyEvent {
    #[strum(serialize = "node-message-update")]
    #[serde(rename = "node-message-update")]
    NodeMessageUpdate(NodeMessage), // 节点消息
    #[strum(serialize = "strategy-data-update")]
    #[serde(rename = "strategy-data-update")]
    StrategyDataUpdate(StrategyData), // 策略数据
}

impl From<StrategyEvent> for Event {
    fn from(event: StrategyEvent) -> Self {
        Event::Strategy(event)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyData {
    pub strategy_id: i32,
    pub data: HashMap<String, Vec<Vec<f64>>>, // cache_key(string) -> cache_value(Vec<f64>)
    pub timestamp: i64,
}







