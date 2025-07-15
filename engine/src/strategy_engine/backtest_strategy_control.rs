use super::StrategyEngineContext;


use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategy;
use types::custom_type::StrategyId;
use types::cache::Key;
use types::strategy::TradeMode;

/* 
    回测策略控制
*/
impl StrategyEngineContext {
    pub async fn backtest_strategy_init(&mut self, strategy_id: i32) -> Result<StrategyId, String> {
        // 判断策略是否在回测策略列表中
        if self.backtest_strategy_list.contains_key(&strategy_id) {
            tracing::warn!("策略已存在, 不进行初始化");
            return Ok(strategy_id);
        }

        let strategy_info = self.get_strategy_info_by_id(strategy_id).await?;
        let strategy_id = strategy_info.id;
        let mut strategy = BacktestStrategy::new(
            strategy_info,
            self.event_publisher.clone(),
            self.command_publisher.clone(),
            self.command_receiver.clone(),
            self.market_event_receiver.resubscribe(),
            self.response_event_receiver.resubscribe(),
            self.database.clone(),
            self.heartbeat.clone()
        ).await;

        strategy.init_strategy().await?;
        self.backtest_strategy_list.insert(strategy_id, strategy);
        Ok(strategy_id)
    }

    // 停止回测策略
    pub async fn backtest_strategy_stop(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance_mut(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.stop_strategy().await.unwrap();
            self.remove_strategy_instance(TradeMode::Backtest, strategy_id).await?;
        }
        Ok(())
    }



    // 播放回测策略
    pub async fn backtest_strategy_play(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance_mut(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.play().await.unwrap();
        }
        Ok(())
    }

    // 重置回测策略
    pub async fn backtest_strategy_reset(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance_mut(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.reset().await.unwrap();
        }
        Ok(())
    }

    // 暂停回测策略
    pub async fn backtest_strategy_pause(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_backtest_strategy_instance_mut(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.pause().await.unwrap();
        }
        Ok(())
    }

    // 播放单根k线
    pub async fn backtest_strategy_play_one_kline(&mut self, strategy_id: i32) -> Result<u32, String> {
        let strategy = self.get_backtest_strategy_instance_mut(strategy_id).await;
        if let Ok(strategy) = strategy {
            let played_signal_count = strategy.play_one_kline().await.unwrap();
            Ok(played_signal_count)
        } else {
            Err("播放单根k线失败".to_string())
        }
    }

    // 获取回测策略的缓存键
    pub async fn get_backtest_strategy_cache_keys(&self, strategy_id: i32) -> Vec<Key> {
        let strategy = self.get_backtest_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.get_context().read().await.get_cache_keys().await
        } else {
            Vec::new()
        }
    }



}