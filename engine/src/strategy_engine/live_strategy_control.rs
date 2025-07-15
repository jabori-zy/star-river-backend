use super::StrategyEngineContext;
use crate::strategy_engine::strategy::live_strategy::LiveStrategy;
use types::custom_type::StrategyId;
use types::cache::Key;
use types::strategy::TradeMode;


impl StrategyEngineContext {

    pub async fn live_strategy_init(&mut self, strategy_id: i32) -> Result<StrategyId, String> {
        // 判断策略是否在回测策略列表中
        if self.live_strategy_list.contains_key(&strategy_id) {
            tracing::warn!("策略已存在, 不进行初始化");
            return Ok(strategy_id);
        }

        let strategy_info = self.get_strategy_info_by_id(strategy_id).await?;
        let strategy_id = strategy_info.id;
        let mut strategy = LiveStrategy::new(
            strategy_info, 
            self.event_publisher.clone(),
            self.command_publisher.clone(),
            self.command_receiver.clone(),
            self.market_event_receiver.resubscribe(), 
            self.response_event_receiver.resubscribe(),
            self.exchange_engine.clone(),
            self.database.clone(),
            self.heartbeat.clone()
        ).await;

        strategy.init_strategy().await?;
        self.live_strategy_list.insert(strategy_id, strategy);
        Ok(strategy_id)
    }

    // 启动实盘策略
    pub async fn live_strategy_start(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_live_strategy_instance_mut(strategy_id).await;
        match strategy {
            Ok(strategy) => {
                strategy.start_strategy().await.unwrap();
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }


    // 实盘策略停止
    pub async fn live_strategy_stop(&mut self, strategy_id: i32) -> Result<(), String> {


        let strategy = self.live_strategy_list.get_mut(&strategy_id).unwrap();
        strategy.stop_strategy().await?;
        self.remove_strategy_instance(TradeMode::Live, strategy_id).await?;


        Ok(())
    }

    pub async fn enable_live_strategy_data_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_live_strategy_instance_mut(strategy_id).await;
        match strategy {
            Ok(strategy) => {
                strategy.enable_strategy_data_push().await.unwrap();
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn disable_live_strategy_data_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.get_live_strategy_instance_mut(strategy_id).await;
        match strategy {
            Ok(strategy) => {
                strategy.disable_strategy_data_push().await.unwrap();
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    // 获取实盘策略的缓存键
    pub async fn get_live_strategy_cache_keys(&self, strategy_id: i32) -> Vec<Key> {
        let strategy = self.get_live_strategy_instance(strategy_id).await;
        if let Ok(strategy) = strategy {
            strategy.get_context().read().await.get_cache_keys().await
        } else {
            Vec::new()
        }
    }
    
}