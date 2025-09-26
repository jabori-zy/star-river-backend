

use crate::{Engine, EngineContext};
use event_center::communication::engine::EngineCommand;
use event_center::event::Event;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct LiveStrategyEngine {
    // 简化的上下文，付费版本可以有更复杂的实现
    context: Arc<RwLock<Box<dyn EngineContext>>>,
}

impl LiveStrategyEngine {
    pub fn new() -> Self {
        // 创建一个空的上下文实现
        let context = Arc::new(RwLock::new(Box::new(DummyEngineContext) as Box<dyn EngineContext>));
        Self { context }
    }
}

impl Engine for LiveStrategyEngine {
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

// 简单的空实现，付费版本可以有完整实现
#[derive(Debug)]
pub struct DummyEngineContext;

#[async_trait::async_trait]
impl EngineContext for DummyEngineContext {
    fn get_engine_name(&self) -> star_river_core::engine::EngineName {
        star_river_core::engine::EngineName::LiveStrategyEngine
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(DummyEngineContext)
    }

    async fn handle_event(&mut self, _event: Event) {
        // 空实现，付费版本可以有实际逻辑
    }

    async fn handle_command(&mut self, _command: EngineCommand) {
        // 空实现，付费版本可以有实际逻辑
    }
}