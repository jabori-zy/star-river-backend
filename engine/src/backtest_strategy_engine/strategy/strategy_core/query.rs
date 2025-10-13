use super::{
    BacktestStrategy, BacktestStrategyError, StatsSnapshot, DateTimeUtc,
    StrategyRunningLogEvent, VirtualOrder, VirtualPosition, VirtualTransaction,
};
use star_river_core::custom_type::PlayIndex;
use star_river_core::key::Key;

impl BacktestStrategy {
    pub async fn get_play_index(&self) -> i32 {
        let context_guard = self.context.read().await;
        context_guard.get_play_index().await
    }

    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let context_guard = self.context.read().await;
        context_guard.get_virtual_orders().await
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let context_guard = self.context.read().await;
        context_guard.get_current_positions().await
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let context_guard = self.context.read().await;
        context_guard.get_history_positions().await
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let context_guard = self.context.read().await;
        context_guard.get_transactions().await
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        let context_guard = self.context.read().await;
        context_guard.get_stats_history(play_index).await
    }

    pub async fn get_running_log(&self) -> Vec<StrategyRunningLogEvent> {
        let context_guard = self.context.read().await;
        context_guard.get_running_log().await
    }

    pub async fn get_strategy_data(&self, play_index: PlayIndex, key: Key, limit: Option<i32>) -> Result<Vec<serde_json::Value>, BacktestStrategyError> {
        let context_guard = self.context.read().await;
        context_guard.get_strategy_data(play_index, key, limit).await
    }

    pub async fn get_strategy_data_by_datetime(
        &self,
        key: Key,
        datetime: DateTimeUtc,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, BacktestStrategyError> {
        let context_guard = self.context.read().await;
        context_guard.get_strategy_data_by_datetime(key, datetime, limit).await
    }
}
