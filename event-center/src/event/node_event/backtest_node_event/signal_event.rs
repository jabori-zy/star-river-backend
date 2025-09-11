use super::super::super::strategy_event::StrategyRunningLogEvent;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalEvent {
    LiveConditionMatch(LiveConditionMatchEvent), // 实盘条件匹配
    BacktestConditionMatch(BacktestConditionMatchEvent), // 回测条件匹配
    BacktestConditionNotMatch(BacktestConditionNotMatchEvent), // 回测条件不匹配
    KlinePlayFinished(KlinePlayFinishedEvent),   // k线播放完毕
    KlinePlay(KlinePlayEvent),                   // K线跳动(信号计数:根据这个值去请求缓存的下标)
    ExecuteOver(ExecuteOverEvent),               // 执行完毕
    RunningLog(StrategyRunningLogEvent),         // 回测条件匹配日志
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConditionMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConditionMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConditionNotMatchEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: i32,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlinePlayFinishedEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: i32,
    pub message_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub play_index: PlayIndex,
    pub timestamp: i64,
}
