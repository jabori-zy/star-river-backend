pub mod backtest_strategy_event;


use types::strategy::node_event::BacktestNodeEvent;
use serde::{Serialize, Deserialize};
use strum::Display;
use std::collections::HashMap;
use crate::Event;
use backtest_strategy_event::BacktestStrategyEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "eventType")]
pub enum StrategyEvent {
    #[strum(serialize = "node-message-update")]
    #[serde(rename = "node-message-update")]
    NodeMessageUpdate(BacktestNodeEvent), // 节点消息
    #[strum(serialize = "live-strategy-data-update")]
    #[serde(rename = "live-strategy-data-update")]
    LiveStrategyDataUpdate(StrategyData), // 实时策略数据更新
    #[strum(serialize = "backtest-strategy-data-update")]
    #[serde(rename = "backtest-strategy-data-update")]
    BacktestStrategyDataUpdate(BacktestStrategyData), // 回测策略数据更新

    #[strum(serialize = "backtest-strategy")]
    #[serde(rename = "backtest-strategy")]
    BacktestStrategy(BacktestStrategyEvent), // 回测策略事件
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







