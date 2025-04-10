pub mod exchange_engine_context;

use std::collections::HashMap;
use std::sync::Arc;
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use exchange_client::ExchangeClient;
use types::market::Exchange;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use std::any::Any;


#[derive(Debug, Clone)]
pub struct ExchangeEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>
}

#[async_trait]
impl Engine for ExchangeEngine {
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

impl ExchangeEngine {
    pub fn new(
        event_publisher: EventPublisher,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let context = ExchangeEngineContext {
            engine_name: EngineName::ExchangeEngine,
            exchanges: HashMap::new(),
            event_publisher,
            event_receiver: vec![response_event_receiver, request_event_receiver],

        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }

    pub async fn is_registered(&self, exchange: &Exchange) -> bool {
        let context_guard = self.context.read().await;
        let exchange_context = context_guard.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();
        exchange_context.is_registered(exchange).await

    }

    pub async fn get_exchange(&self, exchange: &Exchange) -> Box<dyn ExchangeClient> {
        let context_guard = self.context.read().await;
        let exchange_context = context_guard.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();
        let exchanges = exchange_context.get_exchange(exchange).await.unwrap();
        exchanges
    }

    pub async fn get_exchange_mut(&self, exchange: &Exchange) -> Box<dyn ExchangeClient> {
        let mut context_guard = self.context.write().await;
        let exchange_context = context_guard.as_any_mut().downcast_mut::<ExchangeEngineContext>().unwrap();
        let client = exchange_context.get_exchange_mut(exchange).await.unwrap();
        client.clone_box()
    }
}