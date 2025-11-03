use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::collections::HashMap;
use serde_json::Value;
use serde::de::DeserializeOwned;
use crate::state_machine_error::EngineStateMachineError;
use strum::Display;

/// 引擎运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "lowercase")]
pub enum EngineRunState {
    Created,
    Launching,
    Ready,
    Running,
    Stopping,
    Stopped,
    Error,
}

/// 通用引擎动作
#[derive(Debug, Clone, Display)]
#[strum(serialize_all = "lowercase")]
pub enum EngineStateTransTrigger {
    Start,
    StartComplete,
    Stop,
    StopComplete,
    Error(String),
}



pub trait EngineAction: Clone + Debug + Send + Sync + 'static {}




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






/// Generic State Machine - replaces trait objects with generics for zero-cost abstractions
///
/// Type parameters:
/// - `Action`: Action type, must implement Clone + Debug
///
/// State type is fixed to BacktestNodeRunState
/// Event type is fixed to NodeStateTransitionEvent
/// Error type is fixed to BacktestNodeStateMachineError
#[derive(Debug, Clone)]
pub struct EngineStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Current state
    current_state: EngineRunState,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn: fn(&EngineRunState, EngineStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, EngineStateMachineError>,

    /// Engine name for logging and debugging
    engine_name: String,

    /// Optional metadata - stores engine configuration and runtime information
    metadata: Option<Metadata>,
}

/// State change result - contains new state and list of actions to execute
#[derive(Debug, Clone)]
pub struct StateChangeActions<Action>
where
    Action: Clone + Debug,
{
    /// New state after transition
    pub new_state: EngineRunState,

    /// List of actions to execute
    pub actions: Vec<Action>,
}

impl<Action> EngineStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Create a new state machine instance
    ///
    /// # Arguments
    /// - `engine_name`: Name of the engine for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    pub fn new(
        engine_name: String,
        initial_state: EngineRunState,
        transition_fn: fn(&EngineRunState, EngineStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, EngineStateMachineError>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            engine_name,
            metadata: None,
        }
    }

    /// Create a new state machine instance with metadata
    ///
    /// # Arguments
    /// - `engine_name`: Name of the engine for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    /// - `metadata`: Optional metadata
    pub fn with_metadata(
        engine_name: String,
        initial_state: EngineRunState,
        transition_fn: fn(&EngineRunState, EngineStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, EngineStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            engine_name,
            metadata,
        }
    }

    /// Get reference to current state
    pub fn current_state(&self) -> &EngineRunState {
        &self.current_state
    }

    /// Get engine name
    pub fn engine_name(&self) -> &str {
        &self.engine_name
    }

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `event`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(EngineStateMachineError)`: Transition failed
    pub fn transition(&mut self, event: EngineStateTransTrigger) -> Result<StateChangeActions<Action>, EngineStateMachineError> {
        // Call transition function to get new state and actions, passing metadata
        let state_change = (self.transition_fn)(&self.current_state, event, self.metadata.as_ref())?;

        // Update current state
        self.current_state = state_change.new_state.clone();

        Ok(state_change)
    }

    /// Check if currently in specified state
    pub fn is_in_state(&self, state: &EngineRunState) -> bool {
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
    pub fn new(new_state: EngineRunState, actions: Vec<Action>) -> Self {
        Self { new_state, actions }
    }

    /// Get reference to new state
    pub fn new_state(&self) -> &EngineRunState {
        &self.new_state
    }

    /// Get reference to action list
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Consume self and return new state and action list
    pub fn into_parts(self) -> (EngineRunState, Vec<Action>) {
        (self.new_state, self.actions)
    }
}