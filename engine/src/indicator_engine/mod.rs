pub mod indicator_engine_context;
pub mod calculate;
mod talib_bindings;
mod talib;
pub mod indicator_engine_type;
pub mod talib_error;


use std::sync::Arc;
use std::vec;
use event_center::{Event,EventPublisher};
use tokio::sync::RwLock;
use crate::indicator_engine::indicator_engine_context::IndicatorEngineContext;
use tokio::sync::broadcast;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use crate::EngineName;
use std::any::Any;
use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use std::collections::HashMap;
use heartbeat::Heartbeat;
use event_center::{CommandPublisher, CommandReceiver};

#[derive(Debug, Clone)]
pub struct IndicatorEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>
}

#[async_trait]
impl Engine for IndicatorEngine {

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


impl IndicatorEngine {
    pub fn new(
        heartbeat: Arc<Mutex<Heartbeat>>,
        cache_engine: Arc<Mutex<CacheEngine>>,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: CommandReceiver,
        exchange_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let context = IndicatorEngineContext {
            heartbeat,
            cache_engine,
            engine_name: EngineName::IndicatorEngine,
            event_publisher,
            command_publisher,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
            event_receiver: vec![exchange_event_receiver],
            subscribe_indicators: Arc::new(Mutex::new(HashMap::new())),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}
