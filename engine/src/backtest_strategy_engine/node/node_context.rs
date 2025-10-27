use super::node_handles::*;
use crate::backtest_strategy_engine::node::node_state_machine::*;
use async_trait::async_trait;
use event_center::EventPublisher;
use event_center::communication::backtest_strategy::{
    AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand, BacktestNodeCommand, NodeCommandReceiver, NodeCommandSender, StrategyCommandReceiver, StrategyCommandSender
};
use event_center::event::Event;
use event_center::event::EventReceiver;
use event_center::event::node_event::backtest_node_event::common_event::{
    CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload,
};

use event_center::event::node_event::BacktestNodeEvent;
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::strategy::node_benchmark::CompletedCycle;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

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
    // fn get_command_publisher(&self) -> &CommandPublisher {
    //     &self.get_base_context().command_publisher
    // }
    fn get_command_receiver(&self) -> Arc<Mutex<StrategyCommandReceiver>> {
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
    pub event_receivers: Vec<EventReceiver>, // 事件接收器
    // pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
    pub message_receivers: Vec<NodeInputHandle>,
    pub output_handle: HashMap<HandleId, NodeOutputHandle>, // 节点输出句柄
    pub is_enable_event_publish: bool,                      // 是否启用事件发布
    pub state_machine: Box<dyn LiveNodeStateMachine>,       // 状态机
    pub from_node_id: Vec<String>,                          // 来源节点ID
    pub strategy_command_sender: NodeCommandSender,         // 策略命令发送器
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
            // command_publisher: self.command_publisher.clone(),
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
        // command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
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
            // command_publisher,
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

    async fn handle_engine_event(&mut self, event: Event);

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent);

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand);

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

    fn get_strategy_command_sender(&self) -> &StrategyCommandSender {
        &self.get_base_context().strategy_command_sender
    }

    // 获取策略命令接收器
    fn get_node_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.get_base_context().node_command_receiver.clone()
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

    fn get_default_output_handle(&self) -> &NodeOutputHandle;

    // 获取所有输出句柄
    fn get_all_output_handles(&self) -> &HashMap<String, NodeOutputHandle> {
        &self.get_base_context().output_handles
    }

    fn get_output_handle(&self, handle_id: &String) -> &NodeOutputHandle {
        // tracing::info!("get_output_handle: {:?}", handle_id);
        self.get_base_context().output_handles.get(handle_id).unwrap()
    }

    fn get_output_handle_mut(&mut self, handle_id: &String) -> &mut NodeOutputHandle {
        self.get_base_context_mut().output_handles.get_mut(handle_id).unwrap()
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

    fn get_all_input_handles(&self) -> &Vec<NodeInputHandle> {
        &self.get_base_context().input_handles
    }
    fn get_all_input_handles_mut(&mut self) -> &mut Vec<NodeInputHandle> {
        &mut self.get_base_context_mut().input_handles
    }

    fn set_state_machine(&mut self, state_machine: Box<dyn BacktestNodeStateMachine>) {
        self.get_base_context_mut().state_machine = state_machine;
    }

    fn get_play_index(&self) -> PlayIndex {
        *self.get_play_index_watch_rx_ref().borrow()
    }

    async fn send_execute_over_event(&self) {
        // 非叶子节点不发送执行结束事件
        if !self.is_leaf_node() {
            return;
        }
        let payload = ExecuteOverPayload::new(self.get_play_index());
        let execute_over_event: CommonEvent = ExecuteOverEvent::new(
            self.get_node_id().clone(),
            self.get_node_name().clone(),
            self.get_node_id().clone(),
            payload,
        )
        .into();
        let strategy_output_handle = self.get_strategy_output_handle();
        let _ = strategy_output_handle.send(execute_over_event.into());
    }

    async fn send_trigger_event(&self, handle_id: &String) {
        // 叶子节点不发送触发事件
        if self.is_leaf_node() {
            return;
        }
        let payload = TriggerPayload::new(self.get_play_index());
        let trigger_event: CommonEvent =
            TriggerEvent::new(self.get_node_id().clone(), self.get_node_name().clone(), handle_id.clone(), payload).into();
        let output_handle = self.get_output_handle(handle_id);
        output_handle.send(trigger_event.into()).unwrap();
    }

    async fn add_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let payload= AddNodeCycleTrackerCmdPayload::new(node_id.clone(), cycle_tracker);
        let add_node_cycle_tracker_command = AddNodeCycleTrackerCommand::new(node_id, resp_tx, Some(payload)).into();
        let _ = self.get_strategy_command_sender().send(add_node_cycle_tracker_command).await;
        let _ = resp_rx.await.unwrap();
    }

    
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
    pub input_handles: Vec<NodeInputHandle>,                    // 节点事件接收器
    pub output_handles: HashMap<HandleId, NodeOutputHandle>,    // 节点输出句柄
    pub state_machine: Box<dyn BacktestNodeStateMachine>,       // 状态机
    pub from_node_id: Vec<String>,                              // 来源节点ID
    pub strategy_command_sender: StrategyCommandSender,         // 向策略发送命令
    pub node_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 向节点发送命令
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
            input_handles: self.input_handles.clone(),
            output_handles: self.output_handles.clone(),
            state_machine: self.state_machine.clone_box(),
            from_node_id: self.from_node_id.clone(),
            strategy_command_sender: self.strategy_command_sender.clone(),
            node_command_receiver: self.node_command_receiver.clone(),
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
        state_machine: Box<dyn BacktestNodeStateMachine>,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_type,
            is_leaf_node: false,
            output_handles: HashMap::new(),
            cancel_token: CancellationToken::new(),
            input_handles: Vec::new(),
            state_machine,
            from_node_id: Vec::new(),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        }
    }
}
