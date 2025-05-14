use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;
use types::cache::CacheKey;
use types::strategy::node_message::NodeMessage;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio_util::sync::CancellationToken;
use super::strategy_state_machine::StrategyRunState;
use super::strategy_state_machine::StrategyStateMachine;
use tokio::sync::broadcast;
use event_center::Event;

#[async_trait]
pub trait StrategyContext: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn StrategyContext>;
    fn get_strategy_id(&self) -> i32;
    fn get_strategy_name(&self) -> String;
    async fn get_cache_keys(&self) -> Vec<CacheKey>;
    fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle>;
    fn get_cancel_token(&self) -> CancellationToken;
    fn get_state_machine(&self) -> Box<dyn StrategyStateMachine>;
    fn set_state_machine(&mut self, state_machine: Box<dyn StrategyStateMachine>);
    fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>>;
    async fn handle_node_message(&mut self, message: NodeMessage) -> Result<(), String>;
    async fn handle_event(&mut self, event: Event) -> Result<(), String>;
    fn get_run_state(&self) -> StrategyRunState {
        self.get_state_machine().current_state()
    }
}

impl Clone for Box<dyn StrategyContext> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}