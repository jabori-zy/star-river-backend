mod strategy_engine_context;
mod strategy;
pub mod node;
pub mod backtest_strategy_control;
pub mod live_strategy_control;

use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use event_center::EventPublisher;
use tokio::sync::RwLock;
use crate::{exchange_engine::ExchangeEngine, strategy_engine::strategy_engine_context::StrategyEngineContext};
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use std::any::Any;
use heartbeat::Heartbeat;
use types::cache::CacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::TradeMode;

#[derive(Debug, Clone)]
pub struct StrategyEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}

#[async_trait]
impl Engine for StrategyEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>> {
        self.context.clone()
    }
}



impl StrategyEngine{
    pub fn new(
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: CommandReceiver,
        market_event_receiver: EventReceiver,
        request_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let context = StrategyEngineContext {
            engine_name: EngineName::StrategyEngine,
            event_publisher,
            event_receiver: vec![market_event_receiver.resubscribe()],
            command_publisher,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
            database,
            live_strategy_list: HashMap::new(),
            backtest_strategy_list: HashMap::new(),
            market_event_receiver,
            request_event_receiver,
            response_event_receiver,
            exchange_engine,
            heartbeat,
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }

    // 初始化策略
    pub async fn init_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Live => {
                strategy_context.live_strategy_init(strategy_id).await?;
                Ok(())

            }
            TradeMode::Backtest => {
                strategy_context.backtest_strategy_init(strategy_id).await?;
                Ok(())
            }
            _ => {
                Err("不支持的策略类型".to_string())
            }
        }
    }

    // 启动策略
    pub async fn start_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Live => {
                strategy_context.live_strategy_start(strategy_id).await
            }
            _ => {
                Err("不支持的策略类型".to_string())
            }
        }
    }

    // 停止策略
    pub async fn stop_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Live => {
                strategy_context.live_strategy_stop(strategy_id).await
            }
            TradeMode::Backtest => {
                strategy_context.backtest_strategy_stop(strategy_id).await
            }
            _ => {
                Err("不支持的策略类型".to_string())
            }
        }
    }


    // 获取策略缓存键
    pub async fn get_strategy_cache_keys(&mut self, strategy_id: i32) -> Result<Vec<CacheKey>, String> {
        let context = self.context.read().await;
        let strategy_context = context.as_any().downcast_ref::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Live => {
                Ok(strategy_context.get_live_strategy_cache_keys(strategy_id).await)
            }
            TradeMode::Backtest => {
                Ok(strategy_context.get_backtest_strategy_cache_keys(strategy_id).await)
            }
        }
    }

}



// 回测策略控制
impl StrategyEngine {
        // 播放策略
        pub async fn play(&mut self, strategy_id: i32) -> Result<(), String> {
            let mut context = self.context.write().await;
            let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
            strategy_context.backtest_strategy_play(strategy_id).await?;
            Ok(())
        }
    
        // 暂停播放策略
        pub async fn pause(&mut self, strategy_id: i32) -> Result<(), String> {
            let mut context = self.context.write().await;
            let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
            strategy_context.backtest_strategy_pause(strategy_id).await?;
            Ok(())
        }
    
        // 停止播放策略
        pub async fn reset(&mut self, strategy_id: i32) -> Result<(), String> {
            let mut context = self.context.write().await;
            let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
            strategy_context.backtest_strategy_reset(strategy_id).await?;
            Ok(())
        }
    
    
        // 播放单根k线
        pub async fn play_one_kline(&mut self, strategy_id: i32) -> Result<u32, String> {
            let mut context = self.context.write().await;
            let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
            strategy_context.backtest_strategy_play_one_kline(strategy_id).await
        }


}


// 实盘策略控制
impl StrategyEngine {
    // 开启策略数据推送
    pub async fn enable_live_strategy_data_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.enable_live_strategy_data_push(strategy_id).await?;
        Ok(())
    }

    // 关闭策略数据推送
    pub async fn disable_live_strategy_data_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.disable_live_strategy_data_push(strategy_id).await?;
        Ok(())
    }







}