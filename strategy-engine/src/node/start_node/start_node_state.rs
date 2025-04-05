use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use event_center::EventPublisher;
use crate::NodeOutputHandle;
use crate::node::NodeTrait;
use crate::node::start_node::state_machine::StartNodeStateMachine;
use crate::node::NodeStateTransitionEvent;
use crate::node::start_node::state_machine::StartNodeStateAction;
use std::time::Duration;
use crate::node::node_context::{BaseNodeContext, Context};
use crate::node::state_machine::NodeStateMachine;
use std::any::Any;
use event_center::Event;
use crate::*;
use crate::node::node_functions::NodeFunction;
use tokio_util::sync::CancellationToken;


#[derive(Debug, Clone)]
pub struct StartNodeState {
    pub base_state: BaseNodeContext,
    
}


#[async_trait]
impl Context for StartNodeState {

    fn clone_box(&self) -> Box<dyn Context> {
        Box::new(self.clone())
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        tracing::info!("{}: 收到事件: {:?}", self.base_state.node_id, event);
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_state.node_id, message);
        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }


    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_state
    }

    fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {   
        &self.base_state.event_receivers
    }
    fn get_cancel_token(&self) -> &CancellationToken {
        &self.base_state.cancel_token
    }
    fn get_node_id(&self) -> &String {
        &self.base_state.node_id
    }
    fn get_node_name(&self) -> &String {
        &self.base_state.node_name
    }
    fn get_strategy_id(&self) -> &i32 {
        &self.base_state.strategy_id
    }
    fn get_run_state(&self) -> NodeRunState {
        self.base_state.state_machine.current_state()
    }
    fn get_state_machine(&self) -> Box<dyn NodeStateMachine> {
        self.base_state.state_machine.clone_box()
    }
    fn get_enable_event_publish_mut(&mut self) -> &mut bool {
        &mut self.base_state.enable_event_publish
    }
    fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.base_state.output_handle.get(&handle_id).unwrap().sender.clone()
    }


    fn get_output_handle(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.base_state.output_handle
    }
    fn get_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.base_state.output_handle
    }
    fn enable_event_publish(&self) -> &bool {
        &self.base_state.enable_event_publish
    }
    fn get_message_receivers(&self) -> &Vec<NodeMessageReceiver> {
        &self.base_state.message_receivers
    }
    fn get_message_receivers_mut(&mut self) -> &mut Vec<NodeMessageReceiver> {
        &mut self.base_state.message_receivers
    }
    fn set_state_machine(&mut self, state_machine: Box<dyn NodeStateMachine>) {
        let state_machine = state_machine.as_any().downcast_ref::<StartNodeStateMachine>().unwrap().clone();
        self.base_state.state_machine = Box::new(state_machine);
    }


}