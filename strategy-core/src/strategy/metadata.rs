use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use heartbeat::Heartbeat;
use petgraph::{Directed, Graph, graph::NodeIndex};
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId, StrategyName};
use strategy_stats::{StrategyStats, StrategyStatsEvent};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    benchmark::{StrategyBenchmark, strategy_benchmark::StrategyCycleTracker},
    communication::{NodeCommandTrait, StrategyCommandTrait},
    node::NodeTrait,
    strategy::{StrategyConfig, state_machine::StrategyStateMachine},
    variable::{
        custom_variable::CustomVariable,
        sys_varibale::{SysVariable, SysVariableType},
    },
};

#[derive(Debug)]
pub struct StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
    strategy_config: StrategyConfig,
    current_time: Arc<RwLock<DateTime<Utc>>>,
    graph: Graph<N, (), Directed>,
    node_indices: HashMap<NodeId, NodeIndex>,
    database: DatabaseConnection,
    heartbeat: Arc<Mutex<Heartbeat>>,
    cancel_token: CancellationToken,
    state_machine: Arc<RwLock<M>>,
    leaf_node_ids: Vec<NodeId>,
    custom_variable: Arc<RwLock<HashMap<String, CustomVariable>>>,
    sys_variable: Arc<RwLock<HashMap<SysVariableType, SysVariable>>>,
    benchmark: Arc<RwLock<StrategyBenchmark>>,
    cycle_tracker: Arc<RwLock<Option<StrategyCycleTracker>>>,
    strategy_stats: Arc<RwLock<StrategyStats>>,
    strategy_command_transceiver: (mpsc::Sender<X>, Arc<Mutex<mpsc::Receiver<X>>>),
    node_command_sender: HashMap<NodeId, mpsc::Sender<Y>>,
}

impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
    pub fn new(
        mode: &'static str,
        strategy_config: StrategyConfig,
        state_machine: M,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<X>(100);

        let strategy_id = strategy_config.id;
        let strategy_name = strategy_config.name.clone();

        let strategy_stats = Arc::new(RwLock::new(StrategyStats::new(mode, strategy_id)));

        Self {
            strategy_config,
            graph: Graph::new(),
            node_indices: HashMap::new(),
            database,
            heartbeat,
            cancel_token: CancellationToken::new(),
            state_machine: Arc::new(RwLock::new(state_machine)),
            strategy_command_transceiver: (strategy_command_tx, Arc::new(Mutex::new(strategy_command_rx))),
            node_command_sender: HashMap::new(),
            current_time: Arc::new(RwLock::new(Utc::now())),
            leaf_node_ids: Vec::new(),
            custom_variable: Arc::new(RwLock::new(HashMap::new())),
            sys_variable: Arc::new(RwLock::new(HashMap::new())),
            benchmark: Arc::new(RwLock::new(StrategyBenchmark::new(strategy_id, strategy_name))),
            cycle_tracker: Arc::new(RwLock::new(None)),
            strategy_stats,
        }
    }
}

// ============================================================================
// Basic Information Accessors
// ============================================================================
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
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
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
    /// Get reference to state machine
    pub fn state_machine(&self) -> Arc<RwLock<M>> {
        Arc::clone(&self.state_machine)
    }
}

// ============================================================================
// Graph Accessors
// ============================================================================
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
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

    /// Get leaf node ids
    pub fn leaf_node_ids(&self) -> &Vec<NodeId> {
        &self.leaf_node_ids
    }

    /// Set leaf node ids
    pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_ids = leaf_node_ids;
    }
}

// ============================================================================
// Communication Accessors (Command Sender/Receiver)
// ============================================================================
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
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
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
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
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
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
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
    /// Get cancellation token
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}

// ============================================================================
// Other Accessors
// ============================================================================
impl<N, M, X, Y> StrategyMetadata<N, M, X, Y>
where
    N: NodeTrait,
    M: StrategyStateMachine,
    X: StrategyCommandTrait,
    Y: NodeCommandTrait,
{
    /// Get heartbeat
    pub fn heartbeat(&self) -> &Arc<Mutex<Heartbeat>> {
        &self.heartbeat
    }

    /// Get database
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }

    pub async fn current_time(&self) -> DateTime<Utc> {
        *self.current_time.read().await
    }

    pub async fn set_current_time(&self, time: DateTime<Utc>) {
        *self.current_time.write().await = time;
    }
}
