use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use chrono::{DateTime, Utc};
use heartbeat::Heartbeat;
use petgraph::{Directed, Graph, graph::NodeIndex};
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::{CycleId, NodeId, StrategyId, StrategyName};
use strategy_stats::strategy_stats::StrategyStatsAccessor;
// use strategy_stats::StrategyStats;
use tokio::sync::{Mutex, RwLock, mpsc, watch};
use tokio_util::sync::CancellationToken;

use super::leaf_node_execution_tracker::LeafNodeExecutionInfo;
use crate::{
    benchmark::{StrategyBenchmark, strategy_benchmark::StrategyCycleTracker},
    communication::{NodeCommandTrait, StrategyCommandTrait},
    event::node::NodeEventTrait,
    node::NodeTrait,
    strategy::{StrategyConfig, cycle::Cycle, leaf_node_execution_tracker::LeafNodeExecutionTracker, state_machine::StrategyStateMachine},
    variable::{
        custom_variable::CustomVariable,
        sys_varibale::{SysVariable, SysVariableType},
    },
};

#[derive(Debug)]
pub struct StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    cycle_watch_tx: watch::Sender<Cycle>,
    strategy_time_watch_tx: watch::Sender<DateTime<Utc>>,
    strategy_config: StrategyConfig,
    graph: Graph<N, (), Directed>,
    node_indices: HashMap<NodeId, NodeIndex>,
    database: DatabaseConnection,
    heartbeat: Arc<Mutex<Heartbeat>>,
    cancel_token: CancellationToken,
    state_machine: Arc<RwLock<M>>,
    custom_variable: Arc<RwLock<HashMap<String, CustomVariable>>>,
    sys_variable: Arc<RwLock<HashMap<SysVariableType, SysVariable>>>,
    benchmark: Arc<RwLock<StrategyBenchmark>>,
    cycle_tracker: Arc<RwLock<Option<StrategyCycleTracker>>>,
    strategy_stats: S,
    strategy_command_transceiver: (mpsc::Sender<X>, Arc<Mutex<mpsc::Receiver<X>>>),
    node_command_sender: HashMap<NodeId, mpsc::Sender<Y>>,
    leaf_node_execution_tracker: LeafNodeExecutionTracker,
    _phantom: PhantomData<E>,
}

impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    pub fn new(
        strategy_config: StrategyConfig,
        state_machine: M,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_stats: S,
        strategy_time_watch_tx: watch::Sender<DateTime<Utc>>,
    ) -> Self {
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<X>(100);

        let strategy_id = strategy_config.id;
        let strategy_name = strategy_config.name.clone();

        let (cycle_watch_tx, _) = watch::channel::<Cycle>(Cycle::new());
        Self {
            cycle_watch_tx,
            strategy_time_watch_tx,
            strategy_config,
            graph: Graph::new(),
            node_indices: HashMap::new(),
            database,
            heartbeat,
            cancel_token: CancellationToken::new(),
            state_machine: Arc::new(RwLock::new(state_machine)),
            strategy_command_transceiver: (strategy_command_tx, Arc::new(Mutex::new(strategy_command_rx))),
            node_command_sender: HashMap::new(),
            custom_variable: Arc::new(RwLock::new(HashMap::new())),
            sys_variable: Arc::new(RwLock::new(HashMap::new())),
            benchmark: Arc::new(RwLock::new(StrategyBenchmark::new(strategy_id, strategy_name))),
            cycle_tracker: Arc::new(RwLock::new(None)),
            strategy_stats,
            leaf_node_execution_tracker: LeafNodeExecutionTracker::new(),
            _phantom: PhantomData,
        }
    }
}

// ============================================================================
// Basic Information Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    pub fn strategy_config(&self) -> &StrategyConfig {
        &self.strategy_config
    }

    /// Get strategy id
    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_config.id
    }

    /// Get strategy name
    pub fn strategy_name(&self) -> &StrategyName {
        &self.strategy_config.name
    }
}

// ============================================================================
// State Machine Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get reference to state machine
    pub fn state_machine(&self) -> Arc<RwLock<M>> {
        Arc::clone(&self.state_machine)
    }
}

// ============================================================================
// Graph Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_watch_tx.borrow().id()
    }

    pub fn cycle_watch_tx(&self) -> &watch::Sender<Cycle> {
        &self.cycle_watch_tx
    }

    pub fn cycle_watch_rx(&self) -> watch::Receiver<Cycle> {
        self.cycle_watch_tx.subscribe()
    }

    pub fn strategy_time(&self) -> DateTime<Utc> {
        *self.strategy_time_watch_tx.borrow()
    }

    pub fn strategy_time_watch_tx(&self) -> &watch::Sender<DateTime<Utc>> {
        &self.strategy_time_watch_tx
    }

    pub fn strategy_time_watch_rx(&self) -> watch::Receiver<DateTime<Utc>> {
        self.strategy_time_watch_tx.subscribe()
    }

    pub fn strategy_stats(&self) -> &S {
        &self.strategy_stats
    }

    /// Get reference to graph
    pub fn graph(&self) -> &Graph<N, (), Directed> {
        &self.graph
    }

    /// Get mutable reference to graph
    pub fn graph_mut(&mut self) -> &mut Graph<N, (), Directed> {
        &mut self.graph
    }

    /// Get node indices mapping
    pub fn node_indices(&self) -> &HashMap<NodeId, NodeIndex> {
        &self.node_indices
    }

    /// Get mutable reference to node indices
    pub fn node_indices_mut(&mut self) -> &mut HashMap<NodeId, NodeIndex> {
        &mut self.node_indices
    }

    /// Get node index by node id
    pub fn get_node_index(&self, node_id: &str) -> Option<&NodeIndex> {
        self.node_indices.get(node_id)
    }

    /// Add node index mapping
    pub fn add_node_index(&mut self, node_id: NodeId, node_index: NodeIndex) {
        self.node_indices.insert(node_id, node_index);
    }

    pub fn leaf_node_execution_tracker(&self) -> &LeafNodeExecutionTracker {
        &self.leaf_node_execution_tracker
    }

    pub fn leaf_node_execution_tracker_mut(&mut self) -> &mut LeafNodeExecutionTracker {
        &mut self.leaf_node_execution_tracker
    }

    /// Set leaf node ids
    pub async fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_execution_tracker.set_leaf_node_ids(leaf_node_ids);
    }

    pub async fn set_leaf_node_execution_info(&mut self, execution_info: HashMap<NodeId, LeafNodeExecutionInfo>) {
        self.leaf_node_execution_tracker.set_leaf_node_execution_info(execution_info);
    }
}

// ============================================================================
// Communication Accessors (Command Sender/Receiver)
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get strategy command sender
    pub fn strategy_command_sender(&self) -> &mpsc::Sender<X> {
        &self.strategy_command_transceiver.0
    }

    /// Get strategy command receiver
    pub fn strategy_command_receiver(&self) -> &Arc<Mutex<mpsc::Receiver<X>>> {
        &self.strategy_command_transceiver.1
    }

    /// Get node command sender
    pub fn node_command_sender(&self, node_id: &NodeId) -> &mpsc::Sender<Y> {
        &self.node_command_sender[node_id]
    }

    pub fn add_node_command_sender(&mut self, node_id: NodeId, node_command_sender: mpsc::Sender<Y>) {
        self.node_command_sender.insert(node_id, node_command_sender);
    }
}

// ============================================================================
// Variable Management Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get custom variables
    pub fn custom_variable(&self) -> Arc<RwLock<HashMap<String, CustomVariable>>> {
        Arc::clone(&self.custom_variable)
    }

    /// Get system variables
    pub fn sys_variable(&self) -> Arc<RwLock<HashMap<SysVariableType, SysVariable>>> {
        Arc::clone(&self.sys_variable)
    }
}

// ============================================================================
// Benchmark Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get benchmark
    pub fn benchmark(&self) -> &Arc<RwLock<StrategyBenchmark>> {
        &self.benchmark
    }

    /// Get cycle tracker
    pub fn cycle_tracker(&self) -> &Arc<RwLock<Option<StrategyCycleTracker>>> {
        &self.cycle_tracker
    }

    /// Set cycle tracker
    pub async fn set_cycle_tracker(&mut self, cycle_tracker: Option<StrategyCycleTracker>) {
        *self.cycle_tracker.write().await = cycle_tracker;
    }
}

// ============================================================================
// Cancellation Token Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get cancellation token
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}

// ============================================================================
// Other Accessors
// ============================================================================
impl<N, M, X, Y, E, S> StrategyMetadata<N, M, X, Y, E, S>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
    E: NodeEventTrait,
    S: StrategyStatsAccessor,
{
    /// Get heartbeat
    pub fn heartbeat(&self) -> &Arc<Mutex<Heartbeat>> {
        &self.heartbeat
    }

    /// Get database
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }
}
