use star_river_core::custom_type::{StrategyId, StrategyName};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::{event::StrategyStatsEvent, snapshot::StatsSnapshotHistory};

#[derive(Debug)]
pub struct StrategyStatsMetadata {
    pub strategy_id: StrategyId,
    pub strategy_name: StrategyName,
    pub strategy_stats_event_sender: broadcast::Sender<StrategyStatsEvent>,
    pub cancel_token: CancellationToken,
    pub asset_snapshot_history: StatsSnapshotHistory,
}

impl StrategyStatsMetadata {
    pub fn new(strategy_id: StrategyId, strategy_name: StrategyName) -> Self {
        Self {
            strategy_id,
            strategy_name,
            strategy_stats_event_sender: broadcast::channel(100).0,
            cancel_token: CancellationToken::new(),
            asset_snapshot_history: StatsSnapshotHistory::new(None),
        }
    }
}
