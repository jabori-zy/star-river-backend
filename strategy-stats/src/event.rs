use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::StrategyId;
use strum::Display;
use tokio::sync::broadcast;

use crate::snapshot::StatsSnapshot;

// 策略统计事件发送器
pub type StrategyStatsEventSender = broadcast::Sender<StrategyStatsEvent>;
// 策略统计事件接收器
pub type StrategyStatsEventReceiver = broadcast::Receiver<StrategyStatsEvent>;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
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

impl StrategyStatsUpdatedEvent {
    pub fn new(strategy_id: StrategyId, stats_snapshot: StatsSnapshot, datetime: DateTime<Utc>) -> Self {
        Self {
            strategy_id,
            stats_snapshot,
            datetime,
        }
    }
}
