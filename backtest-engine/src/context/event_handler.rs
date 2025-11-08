use async_trait::async_trait;
use engine_core::context_trait::EngineEventHandler;
use crate::context::BacktestEngineContext;
use engine_core::context_trait::EngineContextTrait;
use event_center_new::{Event, EngineCommand};


#[async_trait]
impl EngineEventHandler for BacktestEngineContext {

    async fn handle_event(&mut self, event: Event) {
        tracing::info!("[{}] received event: {:?}", self.engine_name(), event);
    }
    
    async fn handle_command(&mut self, command: EngineCommand) {
        tracing::info!("[{}] received command: {:?}", self.engine_name(), command);
        
    }

}