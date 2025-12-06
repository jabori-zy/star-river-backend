use star_river_core::error::StarRiverErrorTrait;
use strategy_core::strategy::{StrategyConfig, TradeMode, strategy_trait::StrategyLifecycle};
use tokio::time::Duration;

use super::BacktestEngineContext;
use crate::{
    engine_error::{BacktestEngineError, StrategyIsExistSnafu},
    strategy::BacktestStrategy,
};

impl BacktestEngineContext {
    pub async fn init(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        // Check if already initializing or exists
        if self.initializing_strategies.lock().await.contains(&strategy_id) || self.strategy_list.lock().await.contains_key(&strategy_id) {
            tracing::warn!("Strategy already exists or is being initialized, skipping initialization");
            return Err(StrategyIsExistSnafu { strategy_id }.build());
        }

        // Mark as initializing
        self.initializing_strategies.lock().await.insert(strategy_id);
        let strategy_config: StrategyConfig = self.get_strategy_info_by_id(strategy_id).await?;

        let strategy_list = self.strategy_list.clone();
        let database = self.database.clone();
        let heartbeat = self.heartbeat.clone();
        let initializing_set = self.initializing_strategies.clone();

        tokio::spawn(async move {
            let strategy_id = strategy_config.id;
            let strategy_name = strategy_config.name.clone();
            async {
                let mut strategy = BacktestStrategy::new(strategy_config, database, heartbeat);

                // Sleep for 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Err(e) = strategy.check_strategy().await {
                    e.report_log();
                    return;
                }

                if let Err(e) = strategy.init_strategy().await {
                    e.report_log();
                    return;
                }

                strategy_list.lock().await.insert(strategy_id, strategy);
                tracing::info!("strategy [{}] init success", strategy_name);
            }
            .await;
            // Remove from initializing set regardless of success or failure
            initializing_set.lock().await.remove(&strategy_id);
        });

        Ok(())
    }

    pub async fn stop(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        // Try to access strategy using accessor and execute stop operation
        self.with_strategy_mut_async(strategy_id, |strategy| Box::pin(async move { strategy.stop_strategy().await }))
            .await?
            .map_err(|e| {
                e.report_log();
                e
            })?;

        self.remove_strategy_instance(TradeMode::Backtest, strategy_id).await?;

        Ok(())
    }
}
