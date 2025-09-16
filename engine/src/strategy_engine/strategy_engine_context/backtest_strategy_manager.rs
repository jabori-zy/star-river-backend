use super::StrategyEngineContext;

use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategy;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use database::query::strategy_config_query::StrategyConfigQuery;
use event_center::event::strategy_event::StrategyRunningLogEvent;
use snafu::{Report, ResultExt};
use star_river_core::cache::Key;
use star_river_core::error::engine_error::strategy_engine_error::*;
use star_river_core::error::engine_error::strategy_error::*;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::position::virtual_position::VirtualPosition;
use star_river_core::strategy::TradeMode;
use star_river_core::strategy_stats::StatsSnapshot;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use star_river_core::custom_type::PlayIndex;

/*
    回测策略控制
*/
impl StrategyEngineContext {
    pub async fn backtest_strategy_init(
        &mut self,
        strategy_id: i32,
    ) -> Result<(), StrategyEngineError> {
        // 检查是否已经在初始化或已存在
        if self
            .initializing_strategies
            .lock()
            .await
            .contains(&strategy_id)
            || self
                .backtest_strategy_list
                .lock()
                .await
                .contains_key(&strategy_id)
        {
            tracing::warn!("策略已存在或正在初始化中, 不进行初始化");
            return Err(StrategyIsExistSnafu { strategy_id }.fail()?);
        }

        // 标记为初始化中
        self.initializing_strategies
            .lock()
            .await
            .insert(strategy_id);
        let strategy_config: star_river_core::strategy::StrategyConfig =
            self.get_strategy_info_by_id(strategy_id).await.unwrap();

        let strategy_list = self.backtest_strategy_list.clone();
        let database = self.database.clone();
        let heartbeat = self.heartbeat.clone();
        let initializing_set = self.initializing_strategies.clone();

        tokio::spawn(async move {
            let strategy_id = strategy_config.id;
            let strategy_name = strategy_config.name.clone();
            let result: Result<(), BacktestStrategyError> = async {
                let mut strategy =
                    BacktestStrategy::new(strategy_config, database, heartbeat).await;

                strategy.check_strategy().await?;

                strategy.init_strategy().await?;

                strategy_list.lock().await.insert(strategy_id, strategy);
                tracing::info!("strategy [{}({})] init success", strategy_name, strategy_id);
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

    // 停止回测策略
    pub async fn backtest_strategy_stop(&mut self, strategy_id: i32) -> Result<(), StrategyEngineError> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.stop_strategy().await?;
            self.remove_strategy_instance(TradeMode::Backtest, strategy_id)
                .await?;
        }
        Ok(())
    }

    // 播放回测策略
    pub async fn backtest_strategy_play(&mut self, strategy_id: i32) -> Result<(), StrategyEngineError> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.play().await?;
        }
        Ok(())
    }

    // 重置回测策略
    pub async fn backtest_strategy_reset(&mut self, strategy_id: i32) -> Result<(), StrategyEngineError> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.reset().await?;
        }
        Ok(())
    }

    // 暂停回测策略
    pub async fn backtest_strategy_pause(&mut self, strategy_id: i32) -> Result<(), StrategyEngineError> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.pause().await?;
        }
        Ok(())
    }

    // 播放单根k线
    pub async fn backtest_strategy_play_one_kline(
        &mut self,
        strategy_id: i32,
    ) -> Result<PlayIndex, StrategyEngineError> {
        let mut strategy = self.get_backtest_strategy_instance(strategy_id).await?;
        let play_index = strategy.play_one_kline().await?;
        Ok(play_index)
    }

    // 获取回测策略的缓存键
    pub async fn get_backtest_strategy_keys(&self, strategy_id: i32) -> Vec<Key> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.get_context().read().await.get_keys().await
        } else {
            Vec::new()
        }
    }

    pub async fn get_backtest_strategy_play_index(&self, strategy_id: i32) -> Result<i32, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_play_index().await)
        } else {
            Err("获取回测策略播放索引失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_virtual_orders(
        &self,
        strategy_id: i32,
    ) -> Result<Vec<VirtualOrder>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_virtual_orders().await)
        } else {
            Err("获取回测策略虚拟订单失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_current_positions(
        &self,
        strategy_id: i32,
    ) -> Result<Vec<VirtualPosition>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_current_positions().await)
        } else {
            Err("获取回测策略当前持仓失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_history_positions(
        &self,
        strategy_id: i32,
    ) -> Result<Vec<VirtualPosition>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_history_positions().await)
        } else {
            Err("获取回测策略历史持仓失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_stats_history(
        &self,
        strategy_id: i32,
        play_index: i32,
    ) -> Result<Vec<StatsSnapshot>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_stats_history(play_index).await)
        } else {
            Err("获取回测策略快照历史失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_transactions(
        &self,
        strategy_id: i32,
    ) -> Result<Vec<VirtualTransaction>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_transactions().await)
        } else {
            Err("获取回测策略交易明细失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_status(
        &self,
        strategy_id: i32,
    ) -> Result<String, StrategyEngineError> {
        // 检查是否正在初始化或有策略实例
        let is_initializing = self
            .initializing_strategies
            .lock()
            .await
            .contains(&strategy_id);
        let has_instance = self
            .get_backtest_strategy_instance(strategy_id)
            .await
            .is_ok();
        let strategy_status =
            StrategyConfigQuery::get_strategy_status_by_strategy_id(&self.database, strategy_id)
                .await
                .context(DatabaseSnafu {})?;

        let status = ["initializing", "running", "playing", "ready", "pausing"];
        if is_initializing || has_instance {
            // 正在初始化或有实例，返回数据库中的状态
            Ok(strategy_status)
        }
        // 无实例且未初始化, 但是状态为running，则将状态设为stopped
        else if (!is_initializing && !has_instance)
            && (status.contains(&strategy_status.as_str()))
        {
            // 无实例且未初始化，将状态设为stopped并返回
            StrategyConfigMutation::update_strategy_status(
                &self.database,
                strategy_id,
                "stopped".to_string(),
            )
            .await
            .context(DatabaseSnafu {})?;
            Ok("stopped".to_string())
        } else {
            Ok(strategy_status)
        }
    }

    pub async fn get_backtest_strategy_running_log(
        &self,
        strategy_id: i32,
    ) -> Result<Vec<StrategyRunningLogEvent>, StrategyEngineError> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await?;
        Ok(strategy.get_running_log().await)
    }
}
