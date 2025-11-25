use std::{collections::HashMap, fmt::Debug, sync::Arc};

use chrono::{DateTime, Utc};
use star_river_core::custom_type::{CycleId, NodeId, NodeName, StrategyId};
// third-party
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

use super::{node_state_machine::StateMachine, utils::generate_default_output_handle_id};
use crate::{
    communication::{NodeCommandTrait, StrategyCommandTrait},
    event::node::NodeEventTrait,
    node::{
        NodeType,
        node_handles::{HandleId, NodeInputHandle, NodeOutputHandle},
    },
    strategy::cycle::Cycle,
};

/// M: Node State Machine
/// E: Node Event
/// C: Node Command
/// X: Strategy Command

#[derive(Debug)]
pub struct NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    cycle_watch_rx: watch::Receiver<Cycle>,
    strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
    strategy_id: StrategyId,
    node_id: NodeId,
    node_name: NodeName,
    node_type: NodeType,
    cancel_token: CancellationToken,
    input_handles: Vec<NodeInputHandle<E>>,
    output_handles: HashMap<HandleId, NodeOutputHandle<E>>,
    strategy_bound_handle: NodeOutputHandle<E>,
    state_machine: Arc<RwLock<M>>,
    source_nodes: Vec<NodeId>,
    strategy_command_sender: mpsc::Sender<X>,
    node_command_receiver: Arc<Mutex<mpsc::Receiver<C>>>,
    is_leaf_node: bool,
}

impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    pub fn new(
        cycle: watch::Receiver<Cycle>,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        node_type: NodeType,
        state_machine: M,
        strategy_bound_handle: NodeOutputHandle<E>,
        strategy_command_sender: mpsc::Sender<X>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<C>>>,
    ) -> Self {
        Self {
            cycle_watch_rx: cycle,
            strategy_time_watch_rx,
            strategy_id,
            node_id,
            node_name,
            node_type,
            is_leaf_node: false,
            output_handles: HashMap::new(),
            strategy_bound_handle,
            cancel_token: CancellationToken::new(),
            input_handles: Vec::new(),
            state_machine: Arc::new(RwLock::new(state_machine)),
            source_nodes: Vec::new(),
            strategy_command_sender,
            node_command_receiver,
        }
    }
}

impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_watch_rx.borrow().id()
    }

    pub fn cycle_watch_rx(&self) -> watch::Receiver<Cycle> {
        self.cycle_watch_rx.clone()
    }

    pub fn strategy_time(&self) -> DateTime<Utc> {
        *self.strategy_time_watch_rx.borrow()
    }

    pub fn strategy_time_watch_rx(&self) -> watch::Receiver<DateTime<Utc>> {
        self.strategy_time_watch_rx.clone()
    }

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

    // /// Get strategy name
    // pub fn strategy_name(&self) -> &StrategyName {
    //     &self.strategy_name
    // }
}

// ============================================================================
// State Machine Accessors
// ============================================================================
impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    /// Get immutable reference to state machine
    pub fn state_machine(&self) -> Arc<RwLock<M>> {
        Arc::clone(&self.state_machine)
    }
}

// ============================================================================
// Node Relation Accessors (Source Nodes & Leaf Node)
// ============================================================================
impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
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
impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    /// Get input handles
    pub fn input_handles(&self) -> &[NodeInputHandle<E>] {
        &self.input_handles
    }

    /// Add input handle
    pub fn add_input_handle(&mut self, input_handle: NodeInputHandle<E>) {
        self.input_handles.push(input_handle);
    }

    /// Get output handles
    pub fn output_handles(&self) -> &HashMap<HandleId, NodeOutputHandle<E>> {
        &self.output_handles
    }

    /// Add output handle
    pub fn add_output_handle(&mut self, output_handle: NodeOutputHandle<E>) {
        self.output_handles.insert(output_handle.output_handle_id().clone(), output_handle);
    }

    /// Get strategy output handle
    pub fn strategy_bound_handle(&self) -> &NodeOutputHandle<E> {
        &self.strategy_bound_handle
    }

    /// Subscribe strategy output handle
    pub fn subscribe_strategy_bound_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<E> {
        self.strategy_bound_handle.subscribe(subscriber_id)
    }

    /// Subscribe output handle
    pub fn subscribe_output_handle(&mut self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<E> {
        self.output_handles.get_mut(&handle_id).unwrap().subscribe(subscriber_id)
    }

    /// Get default output handle
    pub fn default_output_handle(&self) -> Option<&NodeOutputHandle<E>> {
        let handle_id = generate_default_output_handle_id(&self.node_id);
        self.output_handles.get(&handle_id)
    }

    /// Check if default output handle exists
    pub fn have_default_output_handle(&self) -> bool {
        let handle_id = generate_default_output_handle_id(&self.node_id);
        self.output_handles.contains_key(&handle_id)
    }
}

// ============================================================================
// Communication Accessors (Command Sender/Receiver)
// ============================================================================
impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    /// Get strategy command sender
    pub fn strategy_command_sender(&self) -> &mpsc::Sender<X> {
        &self.strategy_command_sender
    }

    /// Get node command receiver
    pub fn node_command_receiver(&self) -> Arc<Mutex<mpsc::Receiver<C>>> {
        self.node_command_receiver.clone()
    }
}

// ============================================================================
// Cancellation Token Accessors
// ============================================================================
impl<M, E, C, X> NodeMetadata<M, E, C, X>
where
    M: StateMachine,
    E: NodeEventTrait,
    C: NodeCommandTrait,
    X: StrategyCommandTrait,
{
    /// Get cancellation token
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}
