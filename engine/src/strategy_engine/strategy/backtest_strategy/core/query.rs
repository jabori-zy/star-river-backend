use super::{
    BacktestStrategy, VirtualOrder, VirtualPosition, 
    VirtualTransaction, StatsSnapshot, StrategyRunningLogEvent, 
    BacktestStrategyError, KlineKey,
};
use std::sync::Arc;
use star_river_core::cache::{CacheValue, Key};
use star_river_core::custom_type::PlayIndex;



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

    pub async fn get_strategy_data(&self, play_index: PlayIndex, key: Key) -> Result<Vec<Arc<CacheValue>>, BacktestStrategyError> {
        let context_guard = self.context.read().await;
        context_guard.get_strategy_data(play_index, key).await
    }

}