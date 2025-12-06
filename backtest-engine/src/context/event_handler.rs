use async_trait::async_trait;
use engine_core::context_trait::{EngineContextTrait, EngineEventHandler};
use event_center::{EngineCommand, Event};

use crate::context::BacktestEngineContext;

#[async_trait]
impl EngineEventHandler for BacktestEngineContext {
    async fn handle_event(&mut self, event: Event) {
        tracing::info!("[{}] received event: {:?}", self.engine_name(), event);
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        tracing::info!("[{}] received command: {:?}", self.engine_name(), command);
    }
}
