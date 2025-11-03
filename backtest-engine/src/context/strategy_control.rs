use super::BacktestEngineContext;

use crate::{error::strategy_error::BacktestStrategyError, strategy::BacktestStrategy};
use snafu::Report;
use crate::backtest_engine_error::{BacktestEngineError, StrategyIsExistSnafu};
use star_river_core::strategy::TradeMode;
use tokio::time::Duration;



impl BacktestEngineContext {
    pub async fn init(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        // 检查是否已经在初始化或已存在
        if self.initializing_strategies.lock().await.contains(&strategy_id) || self.strategy_list.lock().await.contains_key(&strategy_id) {
            tracing::warn!("策略已存在或正在初始化中, 不进行初始化");
            return Err(StrategyIsExistSnafu { strategy_id }.fail()?);
        }

        // 标记为初始化中
        self.initializing_strategies.lock().await.insert(strategy_id);
        let strategy_config: star_river_core::strategy::StrategyConfig = self.get_strategy_info_by_id(strategy_id).await.unwrap();

        let strategy_list = self.strategy_list.clone();
        let database = self.database.clone();
        let heartbeat = self.heartbeat.clone();
        let initializing_set = self.initializing_strategies.clone();

        tokio::spawn(async move {
            let strategy_id = strategy_config.id;
            let strategy_name = strategy_config.name.clone();
            let result: Result<(), BacktestStrategyError> = async {
                let mut strategy = BacktestStrategy::new(strategy_config, database, heartbeat).await;

                // 休眠1秒
                tokio::time::sleep(Duration::from_millis(500)).await;

                if let Err(e) = strategy.check_strategy().await {
                    let report = Report::from_error(&e);
                    tracing::error!("{}", report);
                    return Err(e);
                }

                if let Err(e) = strategy.init_strategy().await {
                    let report = Report::from_error(&e);
                    tracing::error!("{}", report);
                    return Err(e);
                }

                strategy_list.lock().await.insert(strategy_id, strategy);
                tracing::info!("strategy [{}] init success", strategy_name);
                Ok(())
            }
            .await;

            if let Err(e) = result {
                // 如果策略初始化失败，则将状态重置为stopped
                let report = Report::from_error(&e);
                tracing::error!("{}", report);
            }

            // 无论成功或失败，都从初始化集合中移除
            initializing_set.lock().await.remove(&strategy_id);
        });

        Ok(())
    }


    pub async fn stop(&mut self, strategy_id: i32) -> Result<(), BacktestEngineError> {
        let strategy = self.get_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.stop_strategy().await?;
            self.remove_strategy_instance(TradeMode::Backtest, strategy_id).await?;
        }
        Ok(())
    }
}