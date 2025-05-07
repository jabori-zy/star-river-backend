pub mod cache_engine_type;
pub mod cache_engine_context;


use std::fmt::Debug;
use cache_engine_context::CacheEngineContext;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::Engine;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::Event;
use std::any::Any;
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct NewCacheEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}


#[async_trait]
impl Engine for NewCacheEngine {
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

impl NewCacheEngine {
    pub fn new(
        event_publisher: EventPublisher,
        exchange_event_receiver: broadcast::Receiver<Event>,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let context = CacheEngineContext {
            engine_name: EngineName::CacheEngine,
            event_publisher,
            event_receiver: vec![exchange_event_receiver, response_event_receiver, request_event_receiver],
            cache: Arc::new(RwLock::new(HashMap::new())),

        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}