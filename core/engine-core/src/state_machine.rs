use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use star_river_core::state_machine::Metadata;
use strum::Display;

use crate::state_machine_error::EngineStateMachineError;

/// Engine run state
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

/// Generic engine action
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

    /// Previous state (initially same as current_state)
    previous_state: EngineRunState,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn:
        fn(&EngineRunState, EngineStateTransTrigger, Option<&Metadata>) -> Result<StateChangeActions<Action>, EngineStateMachineError>,

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
        transition_fn: fn(
            &EngineRunState,
            EngineStateTransTrigger,
            Option<&Metadata>,
        ) -> Result<StateChangeActions<Action>, EngineStateMachineError>,
    ) -> Self {
        Self {
            current_state: initial_state,
            previous_state: initial_state,
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
        transition_fn: fn(
            &EngineRunState,
            EngineStateTransTrigger,
            Option<&Metadata>,
        ) -> Result<StateChangeActions<Action>, EngineStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state,
            previous_state: initial_state,
            transition_fn,
            engine_name,
            metadata,
        }
    }

    /// Get reference to current state
    pub fn current_state(&self) -> &EngineRunState {
        &self.current_state
    }

    /// Get reference to previous state
    pub fn previous_state(&self) -> &EngineRunState {
        &self.previous_state
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

        // Save current state as previous state before updating
        self.previous_state = self.current_state;

        // Update current state
        self.current_state = state_change.new_state;

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
