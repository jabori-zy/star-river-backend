mod transaction_engine_context;


use transaction_engine_context::TransactionEngineContext;
use std::sync::Arc;
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use tokio::sync::broadcast;
use sea_orm::DatabaseConnection;
use crate::EngineName;
use crate::exchange_engine::ExchangeEngine;


#[derive(Debug, Clone)]
pub struct TransactionEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}


#[async_trait]
impl Engine for TransactionEngine {
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


impl TransactionEngine {
    pub fn new(
        event_publisher: EventPublisher,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        order_event_receiver: broadcast::Receiver<Event>,
        database: DatabaseConnection,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
    ) -> Self {
        let context = TransactionEngineContext {
            engine_name: EngineName::TransactionEngine,
            event_publisher,
            event_receiver: vec![request_event_receiver, response_event_receiver, order_event_receiver],
            database,
            exchange_engine,
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
        
    }
}


