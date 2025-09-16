use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;
use super::super::super::strategy_event::StrategyRunningLogEvent;

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum CommonEvent {
    // ConditionMatch(ConditionMatchEvent),       // 回测条件匹配
    Trigger(TriggerEvent), // 触发事件
    // KlinePlayFinished(KlinePlayFinishedEvent), // k线播放完毕
    // KlinePlay(KlinePlayEvent),                 // K线跳动(信号计数:根据这个值去请求缓存的下标)
    ExecuteOver(ExecuteOverEvent),             // 执行完毕
    RunningLog(StrategyRunningLogEvent),        // 运行日志
}

// 类型别名
// pub type ConditionMatchEvent = NodeEvent<ConditionMatchPayload>;
pub type TriggerEvent = NodeEvent<TriggerPayload>;
// pub type KlinePlayFinishedEvent = NodeEvent<KlinePlayFinishedPayload>;
// pub type KlinePlayEvent = NodeEvent<KlinePlayPayload>;
pub type ExecuteOverEvent = NodeEvent<ExecuteOverPayload>;

// 为每个事件定义专门的载荷类型（避免From trait冲突）
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ConditionMatchPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: PlayIndex,
// }

// impl ConditionMatchPayload {
//     pub fn new(play_index: PlayIndex) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,
}

impl TriggerPayload {
    pub fn new(play_index: PlayIndex) -> Self {
        Self { play_index }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KlinePlayFinishedPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: PlayIndex,
// }

// impl KlinePlayFinishedPayload {
//     pub fn new(play_index: PlayIndex) -> Self {
//         Self { play_index }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct KlinePlayPayload {
//     #[serde(rename = "playIndex")]
//     pub play_index: PlayIndex,
// }

// impl KlinePlayPayload {
//     pub fn new(play_index: PlayIndex) -> Self {
//         Self { play_index }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverPayload {
    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,
}

impl ExecuteOverPayload {
    pub fn new(play_index: PlayIndex) -> Self {
        Self { play_index }
    }
}
