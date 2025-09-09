pub mod calculate;
pub mod indicator_engine_context;
pub mod indicator_engine_type;
pub mod talib;
pub mod talib_bindings;
pub mod talib_error;

use crate::cache_engine::CacheEngine;
use crate::indicator_engine::indicator_engine_context::IndicatorEngineContext;
use crate::EngineName;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use heartbeat::Heartbeat;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct IndicatorEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
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
    pub fn new(heartbeat: Arc<Mutex<Heartbeat>>, cache_engine: Arc<Mutex<CacheEngine>>) -> Self {
        let context = IndicatorEngineContext {
            heartbeat,
            cache_engine,
            engine_name: EngineName::IndicatorEngine,
            subscribe_indicators: Arc::new(Mutex::new(HashMap::new())),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }
}
