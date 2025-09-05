use std::collections::HashMap;
use event_center::command::backtest_strategy_command::StrategyCommandReceiver;
use event_center::EventPublisher;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::node::node_state_machine::*;
use tokio::sync::broadcast;
use event_center::Event;
use std::fmt::Debug;
use async_trait::async_trait;
use std::any::Any;
use types::strategy::node_event::BacktestNodeEvent;
use types::strategy::TradeMode;
use super::node_types::*;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::{StrategyInnerEventReceiver, StrategyInnerEventPublisher};
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use tokio::sync::RwLock;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::virtual_trading_system::event::VirtualTradingSystemEventReceiver;
use types::custom_type::PlayIndex;


#[async_trait]
pub trait LiveNodeContextTrait: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn LiveNodeContextTrait>;

    async fn handle_event(&mut self, event: Event) -> Result<(), String>;
    
    async fn handle_message(&mut self, message: BacktestNodeEvent) -> Result<(), String>;

    fn get_base_context(&self) -> &LiveBaseNodeContext;

    fn get_base_context_mut(&mut self) -> &mut LiveBaseNodeContext;

    fn get_from_node_id(&self) -> &Vec<String> {
        &self.get_base_context().from_node_id
    }
    fn get_from_node_id_mut(&mut self) -> &mut Vec<String> {
        &mut self.get_base_context_mut().from_node_id
    }
    
    fn get_node_type(&self) -> &NodeType {
        &self.get_base_context().node_type
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.get_base_context().event_publisher
    }

    fn get_event_receivers(&self) -> &Vec<EventReceiver> {
        &self.get_base_context().event_receivers
    }
    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.get_base_context().command_publisher
    }
    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.get_base_context().command_receiver.clone()
    }
    
    fn get_strategy_command_sender(&self) -> &NodeCommandSender {
        &self.get_base_context().strategy_command_sender
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
    fn get_state_machine(&self) -> Box<dyn LiveNodeStateMachine> {
        self.get_base_context().state_machine.clone_box()
    }
    fn get_run_state(&self) -> LiveNodeRunState {
        self.get_base_context().state_machine.current_state()
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle;

    // 获取所有输出句柄
    fn get_all_output_handle(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.get_base_context().output_handle
    }

    // 获取所有输出句柄
    fn get_all_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.get_base_context_mut().output_handle
    }
    fn get_all_message_senders(&self) -> Vec<broadcast::Sender<BacktestNodeEvent>> {
        self.get_base_context().output_handle.values().map(|handle| handle.node_event_sender.clone()).collect()
    }
    fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<BacktestNodeEvent> {
        self.get_base_context().output_handle.get(&handle_id).unwrap().node_event_sender.clone()
    }
    fn get_message_receivers(&self) -> &Vec<NodeInputHandle> {
        &self.get_base_context().message_receivers
    }
    fn get_message_receivers_mut(&mut self) -> &mut Vec<NodeInputHandle> {
        &mut self.get_base_context_mut().message_receivers
    }
    fn get_enable_event_publish_mut(&mut self) -> &mut bool {
        &mut self.get_base_context_mut().is_enable_event_publish
    }
    fn set_state_machine(&mut self, state_machine: Box<dyn LiveNodeStateMachine>) {
        self.get_base_context_mut().state_machine = state_machine;
    }

    fn set_enable_event_publish(&mut self, is_enable_event_publish: bool) {
        self.get_base_context_mut().is_enable_event_publish = is_enable_event_publish;
    }
    
    fn is_enable_event_publish(&self) -> &bool {
        &self.get_base_context().is_enable_event_publish
    }




}

impl Clone for Box<dyn LiveNodeContextTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub type HandleId = String;

#[derive(Debug)]
pub struct LiveBaseNodeContext {
    pub node_type: NodeType,
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub cancel_token: CancellationToken,
    pub event_publisher: EventPublisher,
    pub event_receivers:Vec<EventReceiver>, // 事件接收器
    pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<CommandReceiver>>,
    pub message_receivers: Vec<NodeInputHandle>,
    pub output_handle: HashMap<HandleId, NodeOutputHandle>, // 节点输出句柄
    pub is_enable_event_publish: bool, // 是否启用事件发布
    pub state_machine: Box<dyn LiveNodeStateMachine>, // 状态机
    pub from_node_id: Vec<String>, // 来源节点ID
    pub strategy_command_sender: NodeCommandSender, // 策略命令发送器
}

impl Clone for LiveBaseNodeContext {
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
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
            strategy_command_sender: self.strategy_command_sender.clone(),
        }
    }
}

impl LiveBaseNodeContext {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String,
        node_type: NodeType,
        event_publisher: EventPublisher,
        event_receivers: Vec<EventReceiver>,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        state_machine: Box<dyn LiveNodeStateMachine>,
        strategy_command_sender: NodeCommandSender,
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
            command_publisher,
            command_receiver,
            state_machine,
            from_node_id: Vec::new(),
            strategy_command_sender,
        }
    }
}






#[async_trait]
pub trait BacktestNodeContextTrait: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait>;

    async fn handle_event(&mut self, event: Event);
    
    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent);

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent);

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand);

    fn get_base_context(&self) -> &BacktestBaseNodeContext;

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext;

    fn get_from_node_id(&self) -> &Vec<String> {
        &self.get_base_context().from_node_id
    }
    fn get_from_node_id_mut(&mut self) -> &mut Vec<String> {
        &mut self.get_base_context_mut().from_node_id
    }
    
    fn get_node_type(&self) -> &NodeType {
        &self.get_base_context().node_type
    }

    fn set_is_leaf_node(&mut self, is_leaf_node: bool) {
        self.get_base_context_mut().is_leaf_node = is_leaf_node;
    }

    fn is_leaf_node(&self) -> bool {
        self.get_base_context().is_leaf_node
    }
    
    fn get_node_command_sender(&self) -> &NodeCommandSender {
        &self.get_base_context().node_command_sender
    }

    // 获取策略命令接收器
    fn get_strategy_command_receiver(&self) -> Arc<Mutex<StrategyCommandReceiver>> {
        self.get_base_context().strategy_command_receiver.clone()
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
    fn get_state_machine(&self) -> Box<dyn BacktestNodeStateMachine> {
        self.get_base_context().state_machine.clone_box()
    }
    fn get_run_state(&self) -> BacktestNodeRunState {
        self.get_base_context().state_machine.current_state()
    }

    fn get_play_index_watch_rx(&self) -> tokio::sync::watch::Receiver<PlayIndex> {
        self.get_base_context().play_index_watch_rx.clone()
    }

    fn get_play_index_watch_rx_ref(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.get_base_context().play_index_watch_rx
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle;

    // 获取所有输出句柄
    fn get_all_output_handles(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.get_base_context().output_handles
    }

    fn get_output_handle(&self, handle_id: &String) -> &NodeOutputHandle {
        // tracing::info!("get_output_handle: {:?}", handle_id);
        self.get_base_context().output_handles.get(handle_id).unwrap()
    }

    fn get_strategy_output_handle(&self) -> &NodeOutputHandle {
        let node_id = self.get_node_id();
        let handle_id = format!("{}_strategy_output", node_id);
        self.get_output_handle(&handle_id)
    }

    // 获取所有输出句柄
    fn get_all_output_handle_mut(&mut self) -> &mut HashMap<String, NodeOutputHandle> {
        &mut self.get_base_context_mut().output_handles
    }

    fn get_all_node_event_senders(&self) -> Vec<broadcast::Sender<BacktestNodeEvent>> {
        self.get_base_context().output_handles.values().map(|handle| handle.node_event_sender.clone()).collect()
    }

    fn get_node_event_sender(&self, handle_id: String) -> broadcast::Sender<BacktestNodeEvent> {
        self.get_base_context().output_handles.get(&handle_id).unwrap().node_event_sender.clone()
    }

    fn get_all_input_handles(&self) -> &Vec<NodeInputHandle> {
        &self.get_base_context().input_handles
    }
    fn get_all_input_handles_mut(&mut self) -> &mut Vec<NodeInputHandle> {
        &mut self.get_base_context_mut().input_handles
    }

    fn get_strategy_inner_event_receiver(&self) -> &StrategyInnerEventReceiver {
        &self.get_base_context().strategy_inner_event_receiver
    }

    fn get_enable_event_publish_mut(&mut self) -> &mut bool {
        &mut self.get_base_context_mut().is_enable_event_publish
    }
    fn set_state_machine(&mut self, state_machine: Box<dyn BacktestNodeStateMachine>) {
        self.get_base_context_mut().state_machine = state_machine;
    }

    fn set_enable_event_publish(&mut self, is_enable_event_publish: bool) {
        self.get_base_context_mut().is_enable_event_publish = is_enable_event_publish;
    }
    
    fn is_enable_event_publish(&self) -> &bool {
        &self.get_base_context().is_enable_event_publish
    }

    fn get_play_index(&self) -> PlayIndex {
        *self.get_play_index_watch_rx_ref().borrow()
    }

    // async fn set_play_index(&mut self, play_index: PlayIndex) {
    //     *self.get_base_context_mut().play_index_watch_rx.write().await = play_index;
    // }




}

impl Clone for Box<dyn BacktestNodeContextTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


#[derive(Debug)]
pub struct BacktestBaseNodeContext {
    pub node_type: NodeType,
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    is_leaf_node: bool, // 是否是叶子节点
    pub cancel_token: CancellationToken,
    // pub event_publisher: EventPublisher,
    // pub event_receivers:Vec<EventReceiver>, // 事件接收器
    // pub command_publisher: CommandPublisher,
    // pub command_receiver: Arc<Mutex<CommandReceiver>>,
    pub input_handles: Vec<NodeInputHandle>, // 节点事件接收器
    pub output_handles: HashMap<HandleId, NodeOutputHandle>, // 节点输出句柄
    pub strategy_inner_event_receiver: StrategyInnerEventReceiver, // 策略内部事件接收器
    pub is_enable_event_publish: bool, // 是否启用事件发布
    pub state_machine: Box<dyn BacktestNodeStateMachine>, // 状态机
    pub from_node_id: Vec<String>, // 来源节点ID
    pub node_command_sender: NodeCommandSender, // 向策略发送命令
    pub strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
    pub play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl Clone for BacktestBaseNodeContext {
    fn clone(&self) -> Self {
        Self {
            node_type: self.node_type.clone(),
            strategy_id: self.strategy_id.clone(),
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            is_leaf_node: self.is_leaf_node.clone(),
            cancel_token: self.cancel_token.clone(),
            // event_publisher: self.event_publisher.clone(),
            input_handles: self.input_handles.clone(),
            // event_receivers: self.event_receivers.iter().map(|receiver| receiver.resubscribe()).collect(),
            output_handles: self.output_handles.clone(),
            is_enable_event_publish: self.is_enable_event_publish.clone(),
            state_machine: self.state_machine.clone_box(),
            from_node_id: self.from_node_id.clone(),
            // command_publisher: self.command_publisher.clone(),
            // command_receiver: self.command_receiver.clone(),
            node_command_sender: self.node_command_sender.clone(),
            strategy_inner_event_receiver: self.strategy_inner_event_receiver.resubscribe(),
            strategy_command_receiver: self.strategy_command_receiver.clone(),
            play_index_watch_rx: self.play_index_watch_rx.clone(),
        }
    }
}

impl BacktestBaseNodeContext {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String,
        node_type: NodeType,
        // event_publisher: EventPublisher,
        // event_receivers: Vec<EventReceiver>,
        // command_publisher: CommandPublisher,
        // command_receiver: Arc<Mutex<CommandReceiver>>,
        state_machine: Box<dyn BacktestNodeStateMachine>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            strategy_id,
            node_id, 
            node_name,
            node_type,
            is_leaf_node: false,
            output_handles: HashMap::new(), 
            // event_publisher,
            is_enable_event_publish: false, 
            cancel_token: CancellationToken::new(), 
            input_handles: Vec::new(),
            // event_receivers,
            // command_publisher,
            // command_receiver,
            state_machine,
            from_node_id: Vec::new(),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        }
    }
}