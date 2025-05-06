use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;
use types::strategy::message::NodeMessage;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio_util::sync::CancellationToken;
use super::strategy_state_machine::StrategyRunState;
use super::strategy_state_machine::StrategyStateMachine;


#[async_trait]
pub trait StrategyContext: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn StrategyContext>;
    fn get_strategy_id(&self) -> i32;
    fn get_strategy_name(&self) -> String;
    fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle>;
    fn get_cancel_token(&self) -> CancellationToken;
    fn get_state_machine(&self) -> Box<dyn StrategyStateMachine>;
    fn set_state_machine(&mut self, state_machine: Box<dyn StrategyStateMachine>);
    async fn handle_node_message(&mut self, message: NodeMessage) -> Result<(), String>;
    fn get_run_state(&self) -> StrategyRunState {
        self.get_state_machine().current_state()
    }
}

impl Clone for Box<dyn StrategyContext> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}