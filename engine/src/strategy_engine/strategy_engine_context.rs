use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use sea_orm::DatabaseConnection;
use database::query::strategy_config_query::StrategyConfigQuery;
use std::collections::HashMap;
use crate::strategy_engine::strategy::strategy_state_machine::StrategyRunState;
use crate::EngineName;
use async_trait::async_trait;
use crate::EngineContext;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use heartbeat::Heartbeat;
use super::strategy::StrategyTrait;
use crate::strategy_engine::strategy::live_strategy::LiveStrategy;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategy;
use types::strategy::{Strategy, TradeMode};
use types::custom_type::StrategyId;
use types::cache::CacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use event_center::command::Command;

#[derive(Debug)]
pub struct StrategyEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<EventReceiver>,
    pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<CommandReceiver>>,
    pub database: DatabaseConnection,
    pub strategy_list: HashMap<StrategyId, Box<dyn StrategyTrait>>, //实现了StrategyTrait的策略
    pub market_event_receiver: broadcast::Receiver<Event>,
    pub request_event_receiver: broadcast::Receiver<Event>,
    pub response_event_receiver: broadcast::Receiver<Event>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}


impl Clone for StrategyEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            strategy_list: self.strategy_list.clone(),
            database: self.database.clone(),
            market_event_receiver: self.market_event_receiver.resubscribe(),
            request_event_receiver: self.request_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            exchange_engine: self.exchange_engine.clone(),
            heartbeat: self.heartbeat.clone(),
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
        }
    }
}


#[async_trait]
impl EngineContext for StrategyEngineContext {

    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.command_publisher
    }

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.command_receiver.clone()
    }
    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_command(&mut self, command: Command) {
        let _command = command;
    }

}

impl StrategyEngineContext {
    
    pub async fn get_strategy(&self, strategy_id: StrategyId) -> Result<Box<dyn StrategyTrait>, String> {
        let strategy = self.strategy_list.get(&strategy_id).map(|strategy| strategy.clone());
        if let Some(strategy) = strategy {
            Ok(strategy)
        } else {
            tracing::error!("策略不存在");
            Err("策略不存在".to_string())
        }
    }

    pub async fn get_strategy_mut(&mut self, strategy_id: StrategyId) -> Result<&mut Box<dyn StrategyTrait>, String> {
        if let Some(strategy) = self.strategy_list.get_mut(&strategy_id) {
            Ok(strategy)
        } else {
            tracing::error!("策略不存在");
            Err("策略不存在".to_string())
        }
    }


    pub async fn get_strategy_info_by_id(&self, id: i32) -> Result<Strategy, String> {
        let strategy = StrategyConfigQuery::get_strategy_by_id(&self.database, id).await.unwrap();
        if let Some(strategy) = strategy {
            Ok(strategy)
        } else {
            tracing::error!("策略信息不存在");
            Err("策略信息不存在".to_string())
        }
    }

    pub async fn load_strategy(&mut self, strategy: Strategy) -> Result<i32, String> {
        match strategy.trade_mode {
            TradeMode::Live => {
                let strategy_id = strategy.id;
                let strategy = LiveStrategy::new(
                    strategy, 
                    self.event_publisher.clone(),
                    self.command_publisher.clone(),
                    self.command_receiver.clone(),
                    self.market_event_receiver.resubscribe(), 
                    self.response_event_receiver.resubscribe(),
                    self.exchange_engine.clone(),
                    self.database.clone(),
                    self.heartbeat.clone()
                ).await;
                self.strategy_list.insert(strategy_id, Box::new(strategy));
                Ok(strategy_id)
            }
            TradeMode::Backtest => {
                let strategy_id = strategy.id;
                let strategy = BacktestStrategy::new(
                    strategy,
                    self.event_publisher.clone(),
                    self.command_publisher.clone(),
                    self.command_receiver.clone(),
                    self.market_event_receiver.resubscribe(),
                    self.response_event_receiver.resubscribe(),
                    self.exchange_engine.clone(),
                    self.database.clone(),
                    self.heartbeat.clone()
                ).await;
                self.strategy_list.insert(strategy_id, Box::new(strategy));
                Ok(strategy_id)
            }
            _ => {
                tracing::error!("不支持的策略类型: {}", strategy.trade_mode);
                Err("不支持的策略类型".to_string())
            }
        }
    }


    pub async fn start_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let strategy = self.strategy_list.get_mut(&strategy_id);
        match strategy {
            Some(strategy) => {
                strategy.start_strategy().await.unwrap();
                Ok(())
            }
            None => {
                Err("策略不存在".to_string())
            }
        }
    }



    // 初始化策略
    pub async fn init_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        // 判断策略是否在列表中、
        if self.strategy_list.contains_key(&strategy_id) {
            tracing::warn!("策略已存在, 不进行初始化");
            return Ok(());
        }
        let strategy_info = self.get_strategy_info_by_id(strategy_id).await?;
        // 加载策略（实例化策略）
        self.load_strategy(
            strategy_info
        ).await?;
        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        // 获取策略的状态
        let strategy_state = strategy.get_state_machine().await.current_state();
        if strategy_state != StrategyRunState::Created {
            tracing::warn!("策略状态不是Created, 不设置策略");
            return Ok(());
        }
        strategy.init_strategy().await.unwrap();
        Ok(())
    }

    pub async fn stop_strategy(&mut self, strategy_id: i32) -> Result<(), String> {


        let strategy = self.strategy_list.get_mut(&strategy_id).unwrap();
        strategy.stop_strategy().await?;
        self.remove_strategy(strategy_id).await;


        Ok(())
    }

    async fn remove_strategy(&mut self, strategy_id: i32) {
        self.strategy_list.remove(&strategy_id);
        tracing::info!("策略实例已停止, 从引擎中移除, 策略名称: {}", strategy_id);
    }

    pub async fn get_strategy_cache_keys(&self, strategy_id: i32) -> Vec<CacheKey> {
        let strategy = self.strategy_list.get(&strategy_id).unwrap();
        let cache_keys = strategy.get_strategy_cache_keys().await;
        cache_keys
    }

}
