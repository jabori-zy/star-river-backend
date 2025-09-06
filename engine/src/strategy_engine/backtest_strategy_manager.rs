use super::StrategyEngineContext;


use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategy;
use types::cache::Key;
use types::strategy::TradeMode;
use types::order::virtual_order::VirtualOrder;
use types::position::virtual_position::VirtualPosition;
use types::strategy_stats::StatsSnapshot;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::error::engine_error::strategy_engine_error::*;
use snafu::Report;
use event_center::strategy_event::backtest_strategy_event::StrategyStateLogEvent;
use types::error::engine_error::strategy_error::BacktestStrategyError;
/* 
    回测策略控制
*/
impl StrategyEngineContext {
    pub async fn backtest_strategy_init(&mut self, strategy_id: i32) -> Result<(), StrategyEngineError> {
        // 判断策略是否在回测策略列表中
        if self.backtest_strategy_list.lock().await.contains_key(&strategy_id) {
            tracing::warn!("策略已存在, 不进行初始化");
            return Err(StrategyIsExistSnafu {
                strategy_id,
            }.fail()?);
        }
        let strategy_config: types::strategy::StrategyConfig = self.get_strategy_info_by_id(strategy_id).await.unwrap();

        let strategy_list = self.backtest_strategy_list.clone();
        let database = self.database.clone();
        let heartbeat = self.heartbeat.clone();

        tokio::spawn(async move {
            let strategy_id = strategy_config.id;
            let strategy_name = strategy_config.name.clone();
            let result: Result<(), BacktestStrategyError> = async {

                let mut strategy = BacktestStrategy::new(
                    strategy_config,
                    database,
                    heartbeat
                ).await;

                strategy.check_strategy().await?;

                strategy.init_strategy().await?;
                
                strategy_list.lock().await.insert(strategy_id, strategy);
                tracing::info!("strategy [{}({})] init success", strategy_name, strategy_id);
                Ok(())
            }.await;

            if let Err(e) = result {
                let report = Report::from_error(&e);
                tracing::error!("{}", report);
            }
        });
        
        Ok(())
    }

    // 停止回测策略
    pub async fn backtest_strategy_stop(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.stop_strategy().await.unwrap();
            self.remove_strategy_instance(TradeMode::Backtest, strategy_id).await?;
        }
        Ok(())
    }



    // 播放回测策略
    pub async fn backtest_strategy_play(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.play().await.unwrap();
        }
        Ok(())
    }

    // 重置回测策略
    pub async fn backtest_strategy_reset(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.reset().await.unwrap();
        }
        Ok(())
    }

    // 暂停回测策略
    pub async fn backtest_strategy_pause(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            strategy.pause().await.unwrap();
        }
        Ok(())
    }

    // 播放单根k线
    pub async fn backtest_strategy_play_one_kline(&mut self, strategy_id: i32) -> Result<i32, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(mut strategy) = strategy {
            let play_index = strategy.play_one_kline().await.unwrap();
            Ok(play_index)
        } else {
            Err("播放单根k线失败".to_string())
        }
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

    pub async fn get_backtest_strategy_virtual_orders(&self, strategy_id: i32) -> Result<Vec<VirtualOrder>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_virtual_orders().await)
        } else {
            Err("获取回测策略虚拟订单失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_current_positions(&self, strategy_id: i32) -> Result<Vec<VirtualPosition>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_current_positions().await)
        } else {
            Err("获取回测策略当前持仓失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_history_positions(&self, strategy_id: i32) -> Result<Vec<VirtualPosition>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_history_positions().await)
        } else {
            Err("获取回测策略历史持仓失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_stats_history(&self, strategy_id: i32, play_index: i32) -> Result<Vec<StatsSnapshot>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_stats_history(play_index).await)
        } else {
            Err("获取回测策略快照历史失败".to_string())
        }
    }

    pub async fn get_backtest_strategy_transactions(&self, strategy_id: i32) -> Result<Vec<VirtualTransaction>, String> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            Ok(strategy.get_transactions().await)
        } else {
            Err("获取回测策略交易明细失败".to_string())
        }
    }

}