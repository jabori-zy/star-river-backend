use std::collections::HashMap;
use crate::{NodeOutputHandle, NodeMessageReceiver};
use event_center::EventPublisher;
use tokio_util::sync::CancellationToken;
use crate::node::state_machine::NodeStateMachine;
use tokio::sync::broadcast;
use event_center::Event;
use std::fmt::Debug;
use crate::NodeMessage;
use async_trait::async_trait;
use std::any::Any;
use crate::NodeRunState;

#[async_trait]
pub trait Context: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn Context>;

    fn get_base_context(&self) -> &BaseNodeContext {
        self.as_any().downcast_ref::<BaseNodeContext>().expect("Failed to downcast to BaseNodeContext")
    }

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        self.as_any_mut().downcast_mut::<BaseNodeContext>().expect("Failed to downcast to BaseNodeContext")
    }

    fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {
        &self.get_base_context().event_receivers
    }
    fn get_cancel_token(&self) -> &CancellationToken {
        &self.get_base_context().cancel_token
    }
    fn get_node_id(&self) -> &String {
        &self.get_base_context().node_id
    }
    fn get_node_name(&self) -> &String {
        &self.get_base_context().node_name
    }
    fn get_strategy_id(&self) -> &i32 {
        &self.get_base_context().strategy_id
    }
    fn get_state_machine(&self) -> Box<dyn NodeStateMachine> {
        self.get_base_context().state_machine.clone_box()
    }
    fn get_run_state(&self) -> NodeRunState {
        self.get_base_context().state_machine.current_state()
    }
    fn get_output_handle(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.get_base_context().output_handle
    }
    fn get_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.get_base_context_mut().output_handle
    }
    fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.get_base_context().output_handle.get(&handle_id).unwrap().sender.clone()
    }
    fn get_message_receivers(&self) -> &Vec<NodeMessageReceiver> {
        &self.get_base_context().message_receivers
    }
    fn get_message_receivers_mut(&mut self) -> &mut Vec<NodeMessageReceiver> {
        &mut self.get_base_context_mut().message_receivers
    }
    fn get_enable_event_publish_mut(&mut self) -> &mut bool {
        &mut self.get_base_context_mut().enable_event_publish
    }
    fn set_state_machine(&mut self, state_machine: Box<dyn NodeStateMachine>) {
        self.get_base_context_mut().state_machine = state_machine;
    }
    fn enable_event_publish(&self) -> &bool {
        &self.get_base_context().enable_event_publish
    }
    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        Ok(())
    }
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        Ok(())
    }
}

impl Clone for Box<dyn Context> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


#[derive(Debug)]
pub struct BaseNodeContext {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub cancel_token: CancellationToken,
    pub event_publisher: EventPublisher,
    pub message_receivers: Vec<NodeMessageReceiver>,
    pub event_receivers:Vec<broadcast::Receiver<Event>>,
    pub output_handle: HashMap<String, NodeOutputHandle>,
    pub enable_event_publish: bool,
    pub state_machine: Box<dyn NodeStateMachine>,
}

impl Clone for BaseNodeContext {
    fn clone(&self) -> Self {
        Self {
            strategy_id: self.strategy_id.clone(),
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            cancel_token: self.cancel_token.clone(),
            event_publisher: self.event_publisher.clone(),
            message_receivers: self.message_receivers.clone(),
            event_receivers: self.event_receivers.iter().map(|receiver| receiver.resubscribe()).collect(),
            output_handle: self.output_handle.clone(),
            enable_event_publish: self.enable_event_publish.clone(),
            state_machine: self.state_machine.clone_box(),
        }
    }
}

impl BaseNodeContext {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String,
        event_publisher: EventPublisher,
        event_receivers: Vec<broadcast::Receiver<Event>>,
        state_manager: Box<dyn NodeStateMachine>,
    ) -> Self {
        Self { 
            strategy_id, 
            node_id, 
            node_name, 
            output_handle: HashMap::new(), 
            event_publisher,
            enable_event_publish: false, 
            cancel_token: CancellationToken::new(), 
            message_receivers: Vec::new(),
            event_receivers,
            state_machine: state_manager,
        }
    }
}






