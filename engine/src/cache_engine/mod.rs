mod cache_engine_context;
mod kline_cache_manager;
mod indicator_cache_manager;
mod cache_engine_type;

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
use crate::cache_engine::cache_engine_type::CacheManager;




#[derive(Debug, Clone)]
pub struct CacheEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,

}

#[async_trait]
impl Engine for CacheEngine {
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

impl CacheEngine {
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
            kline_cache_manager: Arc::new(RwLock::new(CacheManager::new())),
            indicator_cache_manager: Arc::new(RwLock::new(CacheManager::new())),

        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}

