use super::node_handles::*;
use crate::backtest_strategy_engine::node::node_state_machine::*;
use crate::backtest_strategy_engine::node::node_utils::NodeUtils;
use async_trait::async_trait;
use event_center::communication::backtest_strategy::{
    AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand, BacktestNodeCommand, NodeCommandReceiver, StrategyCommandSender
};
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::common_event::{
    CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload,
};

use event_center::event::node_event::BacktestNodeEvent;
use star_river_core::custom_type::{HandleId, NodeId, NodeName, PlayIndex, StrategyId};
use star_river_core::strategy::node_benchmark::CompletedCycle;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tokio::sync::broadcast;


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

    fn add_from_node_id(&mut self, from_node_id: String) {
        self.get_base_context_mut().source_nodes.push(from_node_id);
    }

    fn get_from_node_id(&self) -> &Vec<String> {
        &self.get_base_context().source_nodes
    }
    fn get_from_node_id_mut(&mut self) -> &mut Vec<String> {
        &mut self.get_base_context_mut().source_nodes
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


    fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        self.get_base_context_mut().input_handles.push(input_handle);
    }

    fn add_output_handle(&mut self, handle_id: HandleId, sender: broadcast::Sender<BacktestNodeEvent>) {
        let node_id = self.get_node_id();
        let handle = NodeOutputHandle::new(node_id.clone(), handle_id.clone(), sender);
        self.get_base_context_mut().output_handles.insert(handle_id, handle);
    }

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

    fn get_strategy_output_handle_mut(&mut self) -> &mut NodeOutputHandle {
        let node_id = self.get_node_id();
        let handle_id = format!("{}_strategy_output", node_id);
        self.get_output_handle_mut(&handle_id)
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
        self.get_base_context().get_play_index()
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
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub cancel_token: CancellationToken,
    pub input_handles: Vec<NodeInputHandle>,                    // 节点事件接收器
    pub output_handles: HashMap<HandleId, NodeOutputHandle>,    // 节点输出句柄
    pub strategy_output_handle: NodeOutputHandle,
    pub state_machine: Box<dyn BacktestNodeStateMachine>,       // 状态机
    pub source_nodes: Vec<NodeId>,                            // 源节点ID
    pub strategy_command_sender: StrategyCommandSender,         // 向策略发送命令
    pub node_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 向节点发送命令
    pub play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    pub is_leaf_node: bool, // 是否是叶子节点
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
            strategy_output_handle: self.strategy_output_handle.clone(),
            state_machine: self.state_machine.clone_box(),
            source_nodes: self.source_nodes.clone(),
            strategy_command_sender: self.strategy_command_sender.clone(),
            node_command_receiver: self.node_command_receiver.clone(),
            play_index_watch_rx: self.play_index_watch_rx.clone(),
        }
    }
}

impl BacktestBaseNodeContext {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        node_type: NodeType,
        state_machine: Box<dyn BacktestNodeStateMachine>,
        strategy_output_handle: NodeOutputHandle,
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
            strategy_output_handle,
            cancel_token: CancellationToken::new(),
            input_handles: Vec::new(),
            state_machine,
            source_nodes: Vec::new(),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        }
    }


    /// get node id from base context
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// get node name from base context
    pub fn node_name(&self) -> &NodeName {
        &self.node_name
    }

    /// get node type from base context
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// get strategy id from base context
    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }


    pub fn state_machine(&self) -> &Box<dyn BacktestNodeStateMachine> {
        &self.state_machine
    }

    pub fn state_machine_mut(&mut self) -> &mut Box<dyn BacktestNodeStateMachine> {
        &mut self.state_machine
    }

    pub fn source_nodes(&self) -> &[NodeId] {
        &self.source_nodes
    }

    pub fn strategy_command_sender(&self) -> &StrategyCommandSender {
        &self.strategy_command_sender
    }


    pub fn node_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.node_command_receiver.clone()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }
    

    /// set is leaf node flag in base context
    pub fn set_is_leaf_node(&mut self, is_leaf_node: bool) {
        self.is_leaf_node = is_leaf_node;
    }

    pub fn is_leaf_node(&self) -> bool {
        self.is_leaf_node
    }

    pub fn input_handles(&self) -> &Vec<NodeInputHandle> {
        &self.input_handles
    }

    pub fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        self.input_handles.push(input_handle);
    }

    

    pub fn output_handles(&self) -> &HashMap<HandleId, NodeOutputHandle> {
        &self.output_handles
    }

    pub fn add_output_handle(&mut self, output_handle: NodeOutputHandle) {
        self.output_handles.insert(output_handle.output_handle_id(), output_handle);
    }

    pub fn strategy_output_handle(&self) -> &NodeOutputHandle {
        &self.strategy_output_handle
    }


    pub fn default_output_handle(&self) -> Option<&NodeOutputHandle> {
        let handle_id = NodeUtils::generate_default_output_handle_id(&self.node_id);
        self.output_handles.get(&handle_id)
    }

    pub fn have_default_output_handle(&self) -> bool {
        let handle_id = NodeUtils::generate_default_output_handle_id(&self.node_id);
        self.output_handles.contains_key(&handle_id)
    }

    pub fn get_play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

}


