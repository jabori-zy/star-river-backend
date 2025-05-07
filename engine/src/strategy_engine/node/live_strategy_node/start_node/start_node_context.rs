use crate::strategy_engine::node::node_context::{BaseNodeContext, NodeContext};
use std::any::Any;
use event_center::Event;
use types::strategy::node_message::NodeMessage;
use async_trait::async_trait;
use types::strategy::{LiveStrategyConfig, BacktestConfig, SimulatedConfig};




#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BaseNodeContext,
    pub live_config: LiveStrategyConfig,
    
}


#[async_trait]
impl NodeContext for StartNodeContext {

    fn clone_box(&self) -> Box<dyn NodeContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }
    
    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
    


}