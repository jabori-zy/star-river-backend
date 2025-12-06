// ============================================================================
// Standard library imports
// ============================================================================

use std::{
    cmp::PartialEq,
    fmt::{Debug, Display},
};

use star_river_core::{custom_type::StrategyName, state_machine::Metadata};

// ============================================================================
// External crate imports
// ============================================================================
use crate::error::StrategyStateMachineError;

pub trait StrategyRunState: Debug + Clone + Display + PartialEq + Send + Sync {}

pub trait StrategyStateTransTrigger: Debug + Clone + Send + Sync {}

pub trait StrategyStateAction: Clone + Debug + Display + Send + Sync {}

/// Strategy State Machine trait
///
/// Defines core methods that a strategy state machine must implement
pub trait StrategyStateMachine: Debug + Clone + Send + Sync {
    /// State type
    type State: StrategyRunState;
    /// Action type
    type Action: StrategyStateAction;
    /// State transition trigger type
    type Trigger: StrategyStateTransTrigger;

    /// Get reference to current state
    fn current_state(&self) -> &Self::State;

    /// Get reference to previous state
    fn previous_state(&self) -> &Self::State;

    /// Get strategy name
    fn strategy_name(&self) -> &StrategyName;

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `trans_trigger`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StrategyStateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(StrategyStateMachineError)`: Transition failed
    fn transition(
        &mut self,
        trans_trigger: Self::Trigger,
    ) -> Result<StrategyStateChangeActions<Self::State, Self::Action>, StrategyStateMachineError>;

    /// Check if currently in the specified state
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
// StrategyMetadata Structure Definition
// ============================================================================

/// Generic Strategy State Machine - replaces trait objects with generics for zero-cost abstractions
///
/// Type parameters:
/// - `S`: State type, must implement StrategyRunState
/// - `A`: Action type, must implement StrategyStateAction
/// - `T`: Trigger type, must implement StrategyStateTransTrigger

#[derive(Debug, Clone)]
pub struct GenericStrategyStateMachine<S, A, T>
where
    S: StrategyRunState,
    A: StrategyStateAction,
    T: StrategyStateTransTrigger,
{
    /// Current state
    current_state: S,

    /// Previous state (initially same as current_state)
    previous_state: S,

    /// State transition function - returns new state and actions based on current state, event, and metadata
    /// Uses function pointer to avoid extra heap allocations
    transition_fn: fn(&S, T, StrategyName, Option<&Metadata>) -> Result<StrategyStateChangeActions<S, A>, StrategyStateMachineError>,

    /// Strategy name for logging and debugging
    strategy_name: StrategyName,

    /// Optional metadata - stores strategy configuration and runtime information
    metadata: Option<Metadata>,
}

/// State change result - contains new state and list of actions to execute
#[derive(Debug, Clone)]
pub struct StrategyStateChangeActions<S, A>
where
    S: StrategyRunState,
    A: StrategyStateAction,
{
    /// New state after transition
    pub new_state: S,

    /// List of actions to execute
    pub actions: Vec<A>,
}

impl<S, A, T> GenericStrategyStateMachine<S, A, T>
where
    S: StrategyRunState,
    A: StrategyStateAction,
    T: StrategyStateTransTrigger,
{
    /// Create a new strategy state machine instance
    ///
    /// # Arguments
    /// - `strategy_id`: Strategy ID
    /// - `strategy_name`: Strategy name for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    pub fn new(
        strategy_name: StrategyName,
        initial_state: S,
        transition_fn: fn(&S, T, StrategyName, Option<&Metadata>) -> Result<StrategyStateChangeActions<S, A>, StrategyStateMachineError>,
    ) -> Self {
        Self {
            current_state: initial_state.clone(),
            previous_state: initial_state,
            transition_fn,
            strategy_name,
            metadata: None,
        }
    }

    /// Create a new strategy state machine instance with metadata
    ///
    /// # Arguments
    /// - `strategy_id`: Strategy ID
    /// - `strategy_name`: Strategy name for logging and debugging
    /// - `initial_state`: Initial state
    /// - `transition_fn`: State transition function
    /// - `metadata`: Optional metadata
    pub fn with_metadata(
        strategy_name: StrategyName,
        initial_state: S,
        transition_fn: fn(&S, T, StrategyName, Option<&Metadata>) -> Result<StrategyStateChangeActions<S, A>, StrategyStateMachineError>,
        metadata: Option<Metadata>,
    ) -> Self {
        Self {
            current_state: initial_state.clone(),
            previous_state: initial_state,
            transition_fn,
            strategy_name,
            metadata,
        }
    }
}

/// Implement StrategyStateMachine trait for GenericStrategyStateMachine
impl<S, A, T> StrategyStateMachine for GenericStrategyStateMachine<S, A, T>
where
    S: StrategyRunState,
    A: StrategyStateAction,
    T: StrategyStateTransTrigger,
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

    /// Get strategy name
    fn strategy_name(&self) -> &StrategyName {
        &self.strategy_name
    }

    /// Handle state transition event
    ///
    /// # Arguments
    /// - `trans_trigger`: Event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(StrategyStateChangeActions)`: Transition successful, contains new state and action list
    /// - `Err(StrategyStateMachineError)`: Transition failed
    fn transition(
        &mut self,
        trans_trigger: Self::Trigger,
    ) -> Result<StrategyStateChangeActions<Self::State, Self::Action>, StrategyStateMachineError> {
        // Call transition function to get new state and actions, passing metadata
        let state_change = (self.transition_fn)(
            &self.current_state,
            trans_trigger,
            self.strategy_name.clone(),
            self.metadata.as_ref(),
        )?;

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

impl<S, A> StrategyStateChangeActions<S, A>
where
    S: StrategyRunState,
    A: StrategyStateAction,
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

    /// Get clone of new state
    pub fn get_new_state(&self) -> S {
        self.new_state.clone()
    }

    /// Get clone of action list
    pub fn get_actions(&self) -> Vec<A> {
        self.actions.clone()
    }
}
