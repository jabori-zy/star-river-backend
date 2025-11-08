use crate::custom_type::StrategyId;
use crate::strategy_stats::StatsSnapshot;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::Display;

// 策略统计事件发送器
// pub type StrategyStatsEventSender = broadcast::Sender<StrategyStatsEvent>;
// // 策略统计事件接收器
// pub type StrategyStatsEventReceiver = broadcast::Receiver<StrategyStatsEvent>;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum StrategyStatsEvent {
    StrategyStatsUpdated(StrategyStatsUpdatedEvent), // 策略统计已更新
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatsUpdatedEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: StrategyId,

    #[serde(rename = "statsSnapshot")]
    pub stats_snapshot: StatsSnapshot,

    #[serde(rename = "datetime")]
    pub datetime: DateTime<Utc>,
}
