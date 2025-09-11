pub mod backtest_strategy_event;

use super::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use super::node_event::BacktestNodeEvent;
use crate::Event;
use backtest_strategy_event::BacktestStrategyEvent;
use serde::{Deserialize, Serialize};
use star_river_core::error::error_trait::{Language, StarRiverErrorTrait};
use std::collections::HashMap;
use strum::Display;
use utils::get_utc8_timestamp_millis;
use utoipa::ToSchema;

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

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStateLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "nodeState")]
    pub node_state: String,

    #[serde(rename = "nodeStateAction")]
    pub node_state_action: String,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "errorCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

impl NodeStateLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Info,
            message,
            error_code: None,
            error_code_chain: None,
            timestamp: get_utc8_timestamp_millis(),
        }
    }

    pub fn error(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Error,
            message: error.get_error_message(Language::Chinese),
            error_code: Some(error.error_code().to_string()),
            error_code_chain: Some(error.error_code_chain()),
            timestamp: get_utc8_timestamp_millis(),
        }
    }
}

impl From<NodeStateLogEvent> for BacktestNodeEvent {
    fn from(event: NodeStateLogEvent) -> Self {
        BacktestNodeEvent::KlineNode(KlineNodeEvent::StateLog(event))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
pub enum StrategyRunningLogSource {
    #[strum(serialize = "node")]
    #[serde(rename = "Node")]
    Node,
    #[strum(serialize = "virtual_trading_system")]
    #[serde(rename = "VirtualTradingSystem")]
    VirtualTradingSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
pub enum StrategyRunningLogType {
    #[strum(serialize = "condition_match")]
    #[serde(rename = "ConditionMatch")]
    ConditionMatch,
}

// 策略运行日志
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StrategyRunningLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "source")]
    pub source: StrategyRunningLogSource,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "logType")]
    pub log_type: StrategyRunningLogType,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "detail")]
    pub detail: serde_json::Value,

    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

impl StrategyRunningLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
        current_time: i64,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            source,
            log_level: LogLevel::Info,
            log_type,
            message,
            detail,
            error_code: None,
            error_code_chain: None,
            timestamp: current_time,
        }
    }
}
