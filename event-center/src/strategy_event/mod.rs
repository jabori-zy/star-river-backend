
use types::strategy::node_event::NodeEvent;
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
    NodeMessageUpdate(NodeEvent), // 节点消息
    #[strum(serialize = "live-strategy-data-update")]
    #[serde(rename = "live-strategy-data-update")]
    LiveStrategyDataUpdate(StrategyData), // 实时策略数据更新
    #[strum(serialize = "backtest-strategy-data-update")]
    #[serde(rename = "backtest-strategy-data-update")]
    BacktestStrategyDataUpdate(BacktestStrategyData), // 回测策略数据更新
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestStrategyData {
    pub strategy_id: i32,
    pub cache_key: String,
    pub data: Vec<f64>,
    pub timestamp: i64,
}







