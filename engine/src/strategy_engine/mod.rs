mod strategy_engine_context;
mod strategy;
pub mod node;

use std::collections::HashMap;
use std::{hash::Hash, sync::Arc};
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use crate::{exchange_engine::ExchangeEngine, strategy_engine::strategy_engine_context::StrategyEngineContext};
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use database::entities::strategy_config::Model as StrategyConfig;
use database::query::strategy_config_query::StrategyConfigQuery;
use std::any::Any;
use heartbeat::Heartbeat;


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
        market_event_receiver: broadcast::Receiver<Event>,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        database: DatabaseConnection,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let context = StrategyEngineContext {
            engine_name: EngineName::StrategyEngine,
            event_publisher,
            event_receiver: vec![market_event_receiver.resubscribe(), request_event_receiver.resubscribe(), response_event_receiver.resubscribe()],
            database,
            strategy_list: HashMap::new(),
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

    pub async fn init_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.init_strategy(strategy_id).await
    }

    pub async fn start_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.start_strategy(strategy_id).await
    }

    pub async fn stop_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.stop_strategy(strategy_id).await
    }

    pub async fn enable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.enable_strategy_event_push(strategy_id).await
    }

    pub async fn disable_strategy_event_push(&mut self, strategy_id: i32) -> Result<(), String> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        strategy_context.disable_strategy_event_push(strategy_id).await
    }
    
    
    
    
    
}