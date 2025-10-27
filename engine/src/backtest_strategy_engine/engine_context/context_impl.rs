use super::{
    StrategyEngineContext,EngineContext,EngineName, Event, EngineCommand
};

use async_trait::async_trait;
use std::any::Any;



#[async_trait]
impl EngineContext for StrategyEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        let _command = command;
    }
}