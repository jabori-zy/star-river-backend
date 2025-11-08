use super::BacktestEngineContext;

use crate::strategy_new::BacktestStrategy;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::query::strategy_config_query::StrategyConfigQuery;
use snafu::{Report, ResultExt};
use star_river_core::custom_type::StrategyId;
use strategy_core::strategy::StrategyConfig;
use crate::backtest_engine_error::{
    BacktestEngineError,
    StrategyInstanceNotFoundSnafu,
    StrategyConfigNotFoundSnafu,
    UnsupportedTradeModeSnafu,
    DatabaseSnafu,
};
use strategy_core::strategy::TradeMode;
use tokio::sync::{MutexGuard, OwnedMutexGuard};
use std::collections::HashMap;





impl BacktestEngineContext {
    // pub async fn get_strategy_instance(&self, strategy_id: StrategyId) -> Result<&BacktestStrategy, BacktestEngineError> {
    //     let backtest_strategy_list = self.strategy_list.lock().await;
    //     let strategy = backtest_strategy_list.get(&strategy_id);
    //     if let Some(strategy) = strategy {
    //         Ok(strategy)
    //     } else {
    //         let error = StrategyInstanceNotFoundSnafu { strategy_id }.build();
    //         let report = Report::from_error(&error);
    //         tracing::error!("{}", report);
    //         Err(error)
    //     }
    // }

    pub async fn get_strategy_info_by_id(&self, id: i32) -> Result<StrategyConfig, BacktestEngineError> {
        let strategy = StrategyConfigQuery::get_strategy_by_id(&self.database, id)
            .await
            .context(StrategyConfigNotFoundSnafu { strategy_id: id })?;
        Ok(strategy)
    }



    pub async fn remove_strategy_instance(&mut self, trade_mode: TradeMode, strategy_id: i32) -> Result<(), BacktestEngineError> {
        match trade_mode {
            TradeMode::Backtest => {
                self.strategy_list.lock().await.remove(&strategy_id);
                tracing::info!("backtest strategy [{}] instance is removed", strategy_id);
            }
            _ => {
                tracing::error!("backtest strategy engine not support trade mode: {}", trade_mode);
                return Err(UnsupportedTradeModeSnafu {
                    trade_mode: trade_mode.to_string(),
                }
                .build());
            }
        }
        Ok(())
    }


    // 获取回测策略的缓存键
    // pub async fn get_strategy_keys(&self, strategy_id: i32) -> Result<HashMap<Key, NodeId>, BacktestEngineError> {
    //     let strategy = self.get_strategy_instance(strategy_id).await?;
    //     let keys = strategy.with_ctx_read_async(|ctx| {
    //         Box::pin(async move {
    //             ctx.keys().await
    //         })
    //     }).await;
        
    //     Ok(keys)
    // }


    pub async fn get_strategy_status(&self, strategy_id: i32) -> Result<String, BacktestEngineError> {
        // 检查是否正在初始化或有策略实例
        let is_initializing = self.initializing_strategies.lock().await.contains(&strategy_id);
        let has_instance = self.strategy_list.lock().await.contains_key(&strategy_id);
        let strategy_status = StrategyConfigQuery::get_strategy_status_by_strategy_id(&self.database, strategy_id)
            .await
            .context(DatabaseSnafu {})?;

        let status = ["initializing", "running", "playing", "ready", "pausing"];
        if is_initializing || has_instance {
            // 正在初始化或有实例，返回数据库中的状态
            Ok(strategy_status)
        }
        // 无实例且未初始化, 但是状态为running，则将状态设为stopped
        else if (!is_initializing && !has_instance) && (status.contains(&strategy_status.as_str())) {
            // 无实例且未初始化，将状态设为stopped并返回
            StrategyConfigMutation::update_strategy_status(&self.database, strategy_id, "stopped".to_string())
                .await
                .context(DatabaseSnafu {})?;
            Ok("stopped".to_string())
        } else {
            Ok(strategy_status)
        }
    }
}