// ============================================================================
// 标准库导入
// ============================================================================

use std::{
    cmp::PartialEq,
    collections::HashMap,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

// ============================================================================
// 外部 crate 导入
// ============================================================================
use crate::error::NodeStateMachineError;

pub trait RunState: Debug + Clone + Display + PartialEq + Send + Sync {}

pub trait StateTransTrigger: Debug + Clone + Send + Sync {}

pub trait StateAction: Clone + Debug + Display + Send + Sync {}

/// 状态机 trait
///
/// 定义状态机必须实现的核心方法
pub trait StateMachine: Debug + Clone + Send + Sync {
    /// 状态类型
    type State: RunState;
    /// 动作类型
    type Action: StateAction;
    /// 状态转换触发器类型
    type Trigger: StateTransTrigger;

    /// 获取当前状态的引用
    fn current_state(&self) -> &Self::State;

    /// 获取节点名称
    fn node_name(&self) -> &str;

    /// 处理状态转换事件
    ///
    /// # Arguments
    /// - `trans_trigger`: 触发状态转换的事件
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: 转换成功，包含新状态和动作列表
    /// - `Err(NodeStateMachineError)`: 转换失败
    fn transition(&mut self, trans_trigger: Self::Trigger) -> Result<StateChangeActions<Self::State, Self::Action>, NodeStateMachineError>;

    /// 检查当前是否处于指定状态
    ///
    /// 提供默认实现
    fn is_in_state(&self, state: &Self::State) -> bool {
        self.current_state() == state
    }

    /// 获取元数据的引用
    fn metadata(&self) -> Option<&Metadata>;

    /// 检查是否存在元数据
    ///
    /// 提供默认实现
    fn has_metadata(&self) -> bool {
        self.metadata().is_some()
    }
}

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
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
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
pub struct NodeStateMachine<S, A, T>
where
    S: RunState,
    A: StateAction,
    T: StateTransTrigger,
{
    /// Current state
    current_state: S,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn: fn(&S, T, Option<&Metadata>) -> Result<StateChangeActions<S, A>, NodeStateMachineError>,

    /// Node name for logging and debugging
    node_name: String,

    /// Optional metadata - stores node configuration and runtime information
    metadata: Option<Metadata>,
}

/// State change result - contains new state and list of actions to execute
#[derive(Debug, Clone)]
pub struct StateChangeActions<S, A>
where
    S: RunState,
    A: StateAction,
{
    /// New state after transition
    pub new_state: S,

    /// List of actions to execute
    pub actions: Vec<A>,
}

impl<S, A, T> NodeStateMachine<S, A, T>
where
    S: RunState,
    A: StateAction,
    T: StateTransTrigger,
{
    /// Create a new state machine instance
    ///
    /// # Arguments
    /// - `node_name`: Name of the node for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    pub fn new(
        node_name: String,
        initial_state: S,
        transition_fn: fn(&S, T, Option<&Metadata>) -> Result<StateChangeActions<S, A>, NodeStateMachineError>,
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
        initial_state: S,
        transition_fn: fn(&S, T, Option<&Metadata>) -> Result<StateChangeActions<S, A>, NodeStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            node_name,
            metadata,
        }
    }
}

/// 为 NodeStateMachine 实现 StateMachine trait
impl<S, A, T> StateMachine for NodeStateMachine<S, A, T>
where
    S: RunState,
    A: StateAction,
    T: StateTransTrigger,
{
    type State = S;
    type Action = A;
    type Trigger = T;

    /// Get reference to current state
    fn current_state(&self) -> &Self::State {
        &self.current_state
    }

    /// Get node name
    fn node_name(&self) -> &str {
        &self.node_name
    }

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `trans_trigger`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(NodeStateMachineError)`: Transition failed
    fn transition(&mut self, trans_trigger: Self::Trigger) -> Result<StateChangeActions<Self::State, Self::Action>, NodeStateMachineError> {
        // Call transition function to get new state and actions, passing metadata
        let state_change = (self.transition_fn)(&self.current_state, trans_trigger, self.metadata.as_ref())?;

        // Update current state
        self.current_state = state_change.new_state.clone();

        Ok(state_change)
    }

    /// Get reference to metadata
    fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }
}

impl<S, A> StateChangeActions<S, A>
where
    S: RunState,
    A: StateAction,
{
    /// Create state transition result
    pub fn new(new_state: S, actions: Vec<A>) -> Self {
        Self { new_state, actions }
    }

    /// Get reference to new state
    pub fn new_state(&self) -> &S {
        &self.new_state
    }

    /// Get reference to action list
    pub fn actions(&self) -> &[A] {
        &self.actions
    }

    /// Consume self and return new state and action list
    pub fn into_parts(self) -> (S, Vec<A>) {
        (self.new_state, self.actions)
    }
}
