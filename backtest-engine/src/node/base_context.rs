// std
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

// third-party
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_util::sync::CancellationToken;

// workspace crate
use event_center::{
    communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender},
    event::node_event::BacktestNodeEvent,
};
use star_river_core::custom_type::{HandleId, NodeId, NodeName, PlayIndex, StrategyId};

// current crate
use super::{
    NodeType,
    node_handles::{NodeInputHandle, NodeOutputHandle},
    node_state_machine::NodeStateMachine,
    node_utils::NodeUtils,
};

#[derive(Debug, Clone)]
pub struct NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    // Private fields - access through trait methods only
    strategy_id: StrategyId,
    node_id: NodeId,
    node_name: NodeName,
    node_type: NodeType,
    cancel_token: CancellationToken,
    input_handles: Vec<NodeInputHandle>,
    output_handles: HashMap<HandleId, NodeOutputHandle>,
    strategy_output_handle: NodeOutputHandle,
    state_machine: Arc<RwLock<NodeStateMachine<Action>>>,
    source_nodes: Vec<NodeId>,
    strategy_command_sender: StrategyCommandSender,
    node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    is_leaf_node: bool,
}

// ============================================================================
// Constructor
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        node_type: NodeType,
        state_machine: NodeStateMachine<Action>,
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
            state_machine: Arc::new(RwLock::new(state_machine)),
            source_nodes: Vec::new(),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        }
    }
}

// ============================================================================
// Identity & Basic Information Accessors
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get node id
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Get node name
    pub fn node_name(&self) -> &NodeName {
        &self.node_name
    }

    /// Get node kind
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// Get strategy id
    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }
}

// ============================================================================
// State Machine Accessors
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get immutable reference to state machine
    pub fn state_machine(&self) -> Arc<RwLock<NodeStateMachine<Action>>> {
        Arc::clone(&self.state_machine)
    }
}

// ============================================================================
// Node Relation Accessors (Source Nodes & Leaf Node)
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get source nodes (upstream nodes)
    pub fn source_nodes(&self) -> &[NodeId] {
        &self.source_nodes
    }

    /// Get mutable reference to source nodes (module-private)
    pub(super) fn source_nodes_mut(&mut self) -> &mut Vec<NodeId> {
        &mut self.source_nodes
    }

    /// Check if this is a leaf node
    pub fn is_leaf_node(&self) -> bool {
        self.is_leaf_node
    }

    /// Set leaf node flag
    pub fn set_is_leaf_node(&mut self, is_leaf_node: bool) {
        self.is_leaf_node = is_leaf_node;
    }
}

// ============================================================================
// Handle Management (Input/Output Handles)
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get input handles
    pub fn input_handles(&self) -> &[NodeInputHandle] {
        &self.input_handles
    }

    /// Add input handle
    pub fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        self.input_handles.push(input_handle);
    }

    /// Get output handles
    pub fn output_handles(&self) -> &HashMap<HandleId, NodeOutputHandle> {
        &self.output_handles
    }

    /// Add output handle
    pub fn add_output_handle(&mut self, output_handle: NodeOutputHandle) {
        self.output_handles.insert(output_handle.output_handle_id().clone(), output_handle);
    }

    /// Get strategy output handle
    pub fn strategy_output_handle(&self) -> &NodeOutputHandle {
        &self.strategy_output_handle
    }

    /// Subscribe strategy output handle
    pub fn subscribe_strategy_output_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        self.strategy_output_handle.subscribe(subscriber_id)
    }

    /// Subscribe output handle
    pub fn subscribe_output_handle(&mut self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        self.output_handles.get_mut(&handle_id).unwrap().subscribe(subscriber_id)
    }

    /// Get default output handle
    pub fn default_output_handle(&self) -> Option<&NodeOutputHandle> {
        let handle_id = NodeUtils::generate_default_output_handle_id(&self.node_id);
        self.output_handles.get(&handle_id)
    }

    /// Check if default output handle exists
    pub fn have_default_output_handle(&self) -> bool {
        let handle_id = NodeUtils::generate_default_output_handle_id(&self.node_id);
        self.output_handles.contains_key(&handle_id)
    }
}

// ============================================================================
// Communication Accessors (Command Sender/Receiver)
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get strategy command sender
    pub fn strategy_command_sender(&self) -> &StrategyCommandSender {
        &self.strategy_command_sender
    }

    /// Get node command receiver
    pub fn node_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.node_command_receiver.clone()
    }
}

// ============================================================================
// Cancellation Token Accessors
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get cancellation token
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}

// ============================================================================
// Playback Control Accessors
// ============================================================================
impl<Action> NodeBaseContext<Action>
where
    Action: Clone + Debug,
{
    /// Get play index watch receiver
    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }

    /// Get current play index
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }
}
