// ============================================================================
// 标准库导入
// ============================================================================

use std::collections::HashMap;
use std::fmt::Debug;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use crate::error::node_state_machine_error::BacktestNodeStateMachineError;
use strum::Display;

// ============================================================================
// Metadata 结构定义
// ============================================================================

/// 节点元数据 - 只读的键值存储，用于存储节点的配置和运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    data: HashMap<String, Value>,
}

impl Metadata {
    /// 从 HashMap 创建
    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// 从 JSON 字符串创建
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, Value> = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    /// 获取并反序列化为指定类型
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 获取字符串
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    /// 获取整数
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// 获取浮点数
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key)?.as_f64()
    }

    /// 获取布尔值
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// 检查是否包含某个键
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

// ============================================================================
// 节点状态机定义
// ============================================================================

/// Backtest node run state - shared across all nodes
#[derive(Debug, Clone, PartialEq, Display)]
pub enum NodeRunState {
    #[strum(to_string = "Checking")]
    Checking,

    #[strum(to_string = "Created")]
    Created,

    #[strum(to_string = "Initializing")]
    Initializing,

    #[strum(to_string = "Ready")]
    Ready,

    #[strum(to_string = "Backtesting")]
    Backtesting,

    #[strum(to_string = "BacktestComplete")]
    BacktestComplete,

    #[strum(to_string = "Stopping")]
    Stopping,

    #[strum(to_string = "Stopped")]
    Stopped,

    #[strum(to_string = "Failed")]
    Failed,
}

/// State transition event
#[derive(Debug, Display)]
pub enum NodeStateTransTrigger {
    #[strum(to_string = "StartInit")]
    StartInit,
    #[strum(to_string = "FinishInit")]
    FinishInit,
    #[strum(to_string = "StartStop")]
    StartStop,
    #[strum(to_string = "FinishStop")]
    FinishStop,
    #[strum(to_string = "EncounterError")]
    EncounterError(String),
}

/// Generic State Machine - replaces trait objects with generics for zero-cost abstractions
///
/// Type parameters:
/// - `Action`: Action type, must implement Clone + Debug
///
/// State type is fixed to BacktestNodeRunState
/// Event type is fixed to NodeStateTransitionEvent
/// Error type is fixed to BacktestNodeStateMachineError
#[derive(Debug, Clone)]
pub struct NodeStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Current state
    current_state: NodeRunState,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn: fn(&NodeRunState, NodeStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, BacktestNodeStateMachineError>,

    /// Node name for logging and debugging
    node_name: String,

    /// Optional metadata - stores node configuration and runtime information
    metadata: Option<Metadata>,
}

/// State change result - contains new state and list of actions to execute
#[derive(Debug, Clone)]
pub struct StateChangeActions<Action>
where
    Action: Clone + Debug,
{
    /// New state after transition
    pub new_state: NodeRunState,

    /// List of actions to execute
    pub actions: Vec<Action>,
}

impl<Action> NodeStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Create a new state machine instance
    ///
    /// # Arguments
    /// - `node_name`: Name of the node for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    pub fn new(
        node_name: String,
        initial_state: NodeRunState,
        transition_fn: fn(&NodeRunState, NodeStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, BacktestNodeStateMachineError>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            node_name,
            metadata: None,
        }
    }

    /// Create a new state machine instance with metadata
    ///
    /// # Arguments
    /// - `node_name`: Name of the node for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    /// - `metadata`: Optional metadata
    pub fn with_metadata(
        node_name: String,
        initial_state: NodeRunState,
        transition_fn: fn(&NodeRunState, NodeStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, BacktestNodeStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            node_name,
            metadata,
        }
    }

    /// Get reference to current state
    pub fn current_state(&self) -> &NodeRunState {
        &self.current_state
    }

    /// Get node name
    pub fn node_name(&self) -> &str {
        &self.node_name
    }

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `event`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(BacktestNodeStateMachineError)`: Transition failed
    pub fn transition(&mut self, event: NodeStateTransTrigger) -> Result<StateChangeActions<Action>, BacktestNodeStateMachineError> {
        // Call transition function to get new state and actions, passing metadata
        let state_change = (self.transition_fn)(&self.current_state, event, self.metadata.as_ref())?;

        // Update current state
        self.current_state = state_change.new_state.clone();

        Ok(state_change)
    }

    /// Check if currently in specified state
    pub fn is_in_state(&self, state: &NodeRunState) -> bool {
        &self.current_state == state
    }

    /// Get reference to metadata
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Check if metadata exists
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }
}

impl<Action> StateChangeActions<Action>
where
    Action: Clone + Debug,
{
    /// Create state transition result
    pub fn new(new_state: NodeRunState, actions: Vec<Action>) -> Self {
        Self { new_state, actions }
    }

    /// Get reference to new state
    pub fn new_state(&self) -> &NodeRunState {
        &self.new_state
    }

    /// Get reference to action list
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Consume self and return new state and action list
    pub fn into_parts(self) -> (NodeRunState, Vec<Action>) {
        (self.new_state, self.actions)
    }
}
