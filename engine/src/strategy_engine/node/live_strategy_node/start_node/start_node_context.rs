use crate::strategy_engine::node::node_context::{LiveBaseNodeContext, LiveNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::Event;
use std::any::Any;
use types::strategy::node_event::BacktestNodeEvent;
use types::strategy::LiveStrategyConfig;

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: LiveBaseNodeContext,
    pub live_config: LiveStrategyConfig,
}

#[async_trait]
impl LiveNodeContextTrait for StartNodeContext {
    fn clone_box(&self) -> Box<dyn LiveNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_base_context(&self) -> &LiveBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut LiveBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context
            .output_handle
            .get(&format!("start_node_output"))
            .unwrap()
            .clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
        Ok(())
    }
    async fn handle_message(&mut self, message: BacktestNodeEvent) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
}
