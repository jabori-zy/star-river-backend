mod order_engine_context;

use std::{collections::HashMap, sync::Arc};
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use crate::{exchange_engine::ExchangeEngine, order_engine::order_engine_context::OrderEngineContext};
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use std::any::Any;
use heartbeat::Heartbeat;

#[derive(Debug, Clone)]
pub struct OrderEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}


#[async_trait]
impl Engine for OrderEngine {
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
    async fn start(&self) {
        let engine_name = self.get_engine_name().await;
        tracing::info!("{}已启动", engine_name);
        self.listen_events().await;

        // 注册循环任务
        let context = self.get_context();
        let mut context_guard = context.write().await;
        let order_engine_context = context_guard.as_any_mut().downcast_mut::<OrderEngineContext>().unwrap();
        order_engine_context.monitor_unfilled_orders().await;
    }
}


impl OrderEngine{
    pub fn new(
        event_publisher: EventPublisher,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database:DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self {
        let context = OrderEngineContext {
            engine_name: EngineName::OrderEngine,
            event_publisher,
            event_receiver: vec![response_event_receiver, request_event_receiver],
            exchange_engine,
            unfilled_orders: Arc::new(RwLock::new(HashMap::new())),
            database,
            heartbeat
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }
}
