// third-party
use snafu::ResultExt;

// current crate
use super::{BacktestStrategyContext, BacktestStrategyError, StrategyConfigMutation};
use crate::error::strategy_error::UpdateStrategyStatusFailedSnafu;

impl BacktestStrategyContext {
    pub async fn update_strategy_status(&mut self, status: String) -> Result<(), BacktestStrategyError> {
        let strategy_id = self.strategy_id;
        StrategyConfigMutation::update_strategy_status(&self.database, strategy_id, status)
            .await
            .context(UpdateStrategyStatusFailedSnafu {
                strategy_name: self.strategy_name.clone(),
            })?;
        Ok(())
    }

    pub async fn virtual_trading_system_reset(&self) {
        let mut virtual_trading_system = self.virtual_trading_system.lock().await;
        virtual_trading_system.reset();
    }

    pub async fn strategy_stats_reset(&self) {
        let mut strategy_stats = self.strategy_stats.write().await;
        strategy_stats.clear_asset_snapshots().await;
    }

    pub async fn get_play_index(&self) -> i32 {
        let play_index = self.play_index.read().await;
        *play_index
    }
}
