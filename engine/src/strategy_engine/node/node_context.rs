use std::collections::HashMap;
use event_center::EventPublisher;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::node::node_state_machine::*;
use tokio::sync::broadcast;
use event_center::Event;
use std::fmt::Debug;
use async_trait::async_trait;
use std::any::Any;
use types::strategy::message::NodeMessage;
use types::strategy::TradeMode;
use super::node_types::*;


#[async_trait]
pub trait NodeContext: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn NodeContext>;

    async fn handle_event(&mut self, event: Event) -> Result<(), String>;
    
    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String>;

    fn get_base_context(&self) -> &BaseNodeContext;

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext;

    fn get_from_node_id(&self) -> &Vec<String> {
        &self.get_base_context().from_node_id
    }
    fn get_from_node_id_mut(&mut self) -> &mut Vec<String> {
        &mut self.get_base_context_mut().from_node_id
    }
    
    fn get_node_type(&self) -> &NodeType {
        &self.get_base_context().node_type
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

    // fn get_default_output_handle(&self) -> &NodeOutputHandle;

    // 获取所有输出句柄
    fn get_all_output_handle(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.get_base_context().output_handle
    }

    // 获取所有输出句柄
    fn get_all_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.get_base_context_mut().output_handle
    }
    fn get_all_message_senders(&self) -> Vec<broadcast::Sender<NodeMessage>> {
        self.get_base_context().output_handle.values().map(|handle| handle.message_sender.clone()).collect()
    }
    fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.get_base_context().output_handle.get(&handle_id).unwrap().message_sender.clone()
    }
    fn get_message_receivers(&self) -> &Vec<NodeMessageReceiver> {
        &self.get_base_context().message_receivers
    }
    fn get_message_receivers_mut(&mut self) -> &mut Vec<NodeMessageReceiver> {
        &mut self.get_base_context_mut().message_receivers
    }
    fn get_enable_event_publish_mut(&mut self) -> &mut bool {
        &mut self.get_base_context_mut().is_enable_event_publish
    }
    fn set_state_machine(&mut self, state_machine: Box<dyn NodeStateMachine>) {
        self.get_base_context_mut().state_machine = state_machine;
    }

    fn set_enable_event_publish(&mut self, is_enable_event_publish: bool) {
        self.get_base_context_mut().is_enable_event_publish = is_enable_event_publish;
    }
    
    fn is_enable_event_publish(&self) -> &bool {
        &self.get_base_context().is_enable_event_publish
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.get_base_context().event_publisher
    }


}

impl Clone for Box<dyn NodeContext> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub type HandleId = String;

#[derive(Debug)]
pub struct BaseNodeContext {
    pub node_type: NodeType,
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub cancel_token: CancellationToken,
    pub event_publisher: EventPublisher,
    pub message_receivers: Vec<NodeMessageReceiver>,
    pub event_receivers:Vec<broadcast::Receiver<Event>>, // 事件接收器
    pub output_handle: HashMap<HandleId, NodeOutputHandle>, // 节点输出句柄
    pub is_enable_event_publish: bool, // 是否启用事件发布
    pub state_machine: Box<dyn NodeStateMachine>, // 状态机
    pub from_node_id: Vec<String>, // 来源节点ID
}

impl Clone for BaseNodeContext {
    fn clone(&self) -> Self {
        Self {
            node_type: self.node_type.clone(),
            strategy_id: self.strategy_id.clone(),
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            cancel_token: self.cancel_token.clone(),
            event_publisher: self.event_publisher.clone(),
            message_receivers: self.message_receivers.clone(),
            event_receivers: self.event_receivers.iter().map(|receiver| receiver.resubscribe()).collect(),
            output_handle: self.output_handle.clone(),
            is_enable_event_publish: self.is_enable_event_publish.clone(),
            state_machine: self.state_machine.clone_box(),
            from_node_id: self.from_node_id.clone(),
        }
    }
}

impl BaseNodeContext {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String,
        node_type: NodeType,
        event_publisher: EventPublisher,
        event_receivers: Vec<broadcast::Receiver<Event>>,
        state_machine: Box<dyn NodeStateMachine>,
    ) -> Self {
        Self {
            strategy_id,
            node_id, 
            node_name,
            node_type,
            output_handle: HashMap::new(), 
            event_publisher,
            is_enable_event_publish: false, 
            cancel_token: CancellationToken::new(), 
            message_receivers: Vec::new(),
            event_receivers,
            state_machine,
            from_node_id: Vec::new(),
        }
    }
}






