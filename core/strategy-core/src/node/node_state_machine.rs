// ============================================================================
// Standard Library Imports
// ============================================================================

use std::{
    cmp::PartialEq,
    fmt::{Debug, Display},
};

use star_river_core::state_machine::Metadata;

// ============================================================================
// External Crate Imports
// ============================================================================
use crate::error::NodeStateMachineError;

pub trait RunState: Debug + Clone + Display + PartialEq + Send + Sync {}

pub trait StateTransTrigger: Debug + Clone + Send + Sync {}

pub trait StateAction: Clone + Debug + Display + Send + Sync {}

/// State machine trait
///
/// Defines core methods that must be implemented by state machines
pub trait StateMachine: Debug + Clone + Send + Sync {
    /// State type
    type State: RunState;
    /// Action type
    type Action: StateAction;
    /// State transition trigger type
    type Trigger: StateTransTrigger;

    /// Get reference to current state
    fn current_state(&self) -> &Self::State;

    /// Get reference to previous state
    fn previous_state(&self) -> &Self::State;

    /// Get node name
    fn node_name(&self) -> &str;

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `trans_trigger`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(NodeStateMachineError)`: Transition failed
    fn transition(&mut self, trans_trigger: Self::Trigger) -> Result<StateChangeActions<Self::State, Self::Action>, NodeStateMachineError>;

    /// Check if currently in specified state
    ///
    /// Provides default implementation
    fn is_in_state(&self, state: &Self::State) -> bool {
        self.current_state() == state
    }

    /// Get reference to metadata
    fn metadata(&self) -> Option<&Metadata>;

    /// Check if metadata exists
    ///
    /// Provides default implementation
    fn has_metadata(&self) -> bool {
        self.metadata().is_some()
    }
}

// ============================================================================
// Metadata Structure Definition
// ============================================================================

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

    /// Previous state (initially same as current_state)
    previous_state: S,

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
            current_state: initial_state.clone(),
            previous_state: initial_state,
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
            current_state: initial_state.clone(),
            previous_state: initial_state,
            transition_fn,
            node_name,
            metadata,
        }
    }
}

/// Implement StateMachine trait for NodeStateMachine
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

    /// Get reference to previous state
    fn previous_state(&self) -> &Self::State {
        &self.previous_state
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

        // Save current state as previous state before updating
        self.previous_state = self.current_state.clone();

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
