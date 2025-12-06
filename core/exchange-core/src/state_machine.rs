use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use strum::Display;

use crate::error::ExchangeStateMachineError;

/// Engine runtime state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum ExchangeRunState {
    NotRegistered,
    Created,
    Initializing,
    Connected,
    Stopping,
    Stopped,
    Error,
}

/// Common engine action
#[derive(Debug, Clone, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeStateTransTrigger {
    StartInit,
    FinishInit,
    Shutdown,
    FinishShutdown,
    Error(String),
}

pub trait ExchangeAction: Clone + Debug + Send + Sync + 'static {}

/// Node metadata - Read-only key-value store for node configuration and runtime info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    data: HashMap<String, Value>,
}

impl Metadata {
    /// Create from HashMap
    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, Value> = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    /// Get and deserialize to specified type
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get string value
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    /// Get integer value
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }

    /// Get float value
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.get(key)?.as_f64()
    }

    /// Get boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    /// Check if contains a key
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
pub struct ExchangeStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Current state
    current_state: ExchangeRunState,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn: fn(
        &ExchangeRunState,
        ExchangeStateTransTrigger,
        Option<&Metadata>,
    ) -> Result<StateChangeActions<Action>, ExchangeStateMachineError>,

    /// Exchange name for logging and debugging
    exchange_name: String,

    /// Optional metadata - stores exchange configuration and runtime information
    metadata: Option<Metadata>,
}

/// State change result - contains new state and list of actions to execute
#[derive(Debug, Clone)]
pub struct StateChangeActions<Action>
where
    Action: Clone + Debug,
{
    /// New state after transition
    pub new_state: ExchangeRunState,

    /// List of actions to execute
    pub actions: Vec<Action>,
}

impl<Action> ExchangeStateMachine<Action>
where
    Action: Clone + Debug,
{
    /// Create a new state machine instance
    ///
    /// # Arguments
    /// - `exchange_name`: Name of the exchange for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    pub fn new(
        exchange_name: String,
        initial_state: ExchangeRunState,
        transition_fn: fn(
            &ExchangeRunState,
            ExchangeStateTransTrigger,
            Option<&Metadata>,
        ) -> Result<StateChangeActions<Action>, ExchangeStateMachineError>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            exchange_name,
            metadata: None,
        }
    }

    /// Create a new state machine instance with metadata
    ///
    /// # Arguments
    /// - `exchange_name`: Name of the exchange for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    /// - `metadata`: Optional metadata
    pub fn with_metadata(
        exchange_name: String,
        initial_state: ExchangeRunState,
        transition_fn: fn(
            &ExchangeRunState,
            ExchangeStateTransTrigger,
            Option<&Metadata>,
        ) -> Result<StateChangeActions<Action>, ExchangeStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state,
            transition_fn,
            exchange_name,
            metadata,
        }
    }

    /// Get reference to current state
    pub fn current_state(&self) -> &ExchangeRunState {
        &self.current_state
    }

    /// Get exchange name
    pub fn exchange_name(&self) -> &str {
        &self.exchange_name
    }

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `event`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(ExchangeStateMachineError)`: Transition failed
    pub fn transition(
        &mut self,
        trans_trigger: ExchangeStateTransTrigger,
    ) -> Result<StateChangeActions<Action>, ExchangeStateMachineError> {
        // Call transition function to get new state and actions, passing metadata
        let state_change = (self.transition_fn)(&self.current_state, trans_trigger, self.metadata.as_ref())?;

        // Update current state
        self.current_state = state_change.new_state.clone();

        Ok(state_change)
    }

    /// Check if currently in specified state
    pub fn is_in_state(&self, state: &ExchangeRunState) -> bool {
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
    pub fn new(new_state: ExchangeRunState, actions: Vec<Action>) -> Self {
        Self { new_state, actions }
    }

    /// Get reference to new state
    pub fn new_state(&self) -> &ExchangeRunState {
        &self.new_state
    }

    /// Get reference to action list
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Consume self and return new state and action list
    pub fn into_parts(self) -> (ExchangeRunState, Vec<Action>) {
        (self.new_state, self.actions)
    }
}
