pub mod indicator_engine_context;
mod talib_bindings;
mod talib;

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
        event_publisher: EventPublisher,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let context = IndicatorEngineContext {
            engine_name: EngineName::IndicatorEngine,
            event_publisher,
            event_receiver: vec![response_event_receiver, request_event_receiver],
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}
