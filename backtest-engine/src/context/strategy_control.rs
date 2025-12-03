use snafu::Report;
use star_river_core::error::StarRiverErrorTrait;
use strategy_core::strategy::{StrategyConfig, TradeMode, strategy_trait::StrategyLifecycle};
use tokio::time::Duration;

use super::BacktestEngineContext;
use crate::{
    engine_error::{BacktestEngineError, StrategyIsExistSnafu},
    strategy::{BacktestStrategy, strategy_error::BacktestStrategyError},
};

impl BacktestEngineContext {
    pub async fn init(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        // 检查是否已经在初始化或已存在
        if self.initializing_strategies.lock().await.contains(&strategy_id) || self.strategy_list.lock().await.contains_key(&strategy_id) {
            tracing::warn!("策略已存在或正在初始化中, 不进行初始化");
            return Err(StrategyIsExistSnafu { strategy_id }.build());
        }

        // 标记为初始化中
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

                // 休眠1秒
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Err(e) = strategy.check_strategy().await {
                    e.report();
                    return;
                }

                if let Err(e) = strategy.init_strategy().await {
                    e.report();
                    return;
                }

                strategy_list.lock().await.insert(strategy_id, strategy);
                tracing::info!("strategy [{}] init success", strategy_name);
            }
            .await;
            // 无论成功或失败，都从初始化集合中移除
            initializing_set.lock().await.remove(&strategy_id);
        });

        Ok(())
    }

    pub async fn stop(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        // 尝试使用访问器访问策略并执行停止操作
        self.with_strategy_mut_async(strategy_id, |strategy| Box::pin(async move { strategy.stop_strategy().await }))
            .await?
            .map_err(|e| {
                let report = Report::from_error(&e);
                tracing::error!("{}", report);
                e
            })?;

        self.remove_strategy_instance(TradeMode::Backtest, strategy_id).await?;

        Ok(())
    }
}
