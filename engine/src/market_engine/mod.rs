mod market_engine_context;
mod market_engine_type;
use std::sync::Arc;
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use crate::{exchange_engine::ExchangeEngine, market_engine::market_engine_context::MarketEngineContext};
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use tokio::sync::Mutex;
use std::any::Any;
use std::collections::HashMap;


#[derive(Clone, Debug)]
pub struct MarketEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}

#[async_trait]
impl Engine for MarketEngine {
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


impl MarketEngine{
    pub fn new(
        event_publisher: EventPublisher,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
    ) -> Self {
        let context = MarketEngineContext {
            engine_name: EngineName::MarketEngine,
            event_publisher,
            event_receiver: vec![response_event_receiver, request_event_receiver],
            exchange_engine,
            subscribe_klines: Arc::new(Mutex::new(HashMap::new())),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}

