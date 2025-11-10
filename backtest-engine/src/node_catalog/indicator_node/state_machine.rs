use strategy_core::{
    error::{NodeStateMachineError, node_state_machine_error::NodeTransFailedSnafu},
    node::node_state_machine::{NodeStateMachine, StateAction, StateChangeActions},
};
use strum::Display;

use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};

// ============================================================================
// IndicatorNode State Machine Type Alias
// ============================================================================

/// IndicatorNode state machine type alias
pub type IndicatorNodeStateMachine = NodeStateMachine<NodeRunState, IndicatorNodeAction, NodeStateTransTrigger>;

// ============================================================================
// IndicatorNode Action Definition
// ============================================================================

/// Actions to be executed after IndicatorNode state transitions
#[derive(Debug, Clone, Display)]
pub enum IndicatorNodeAction {
    ListenAndHandleExternalEvents,  // Handle external events
    ListenAndHandleNodeEvents,      // Listen and handle node messages
    ListenAndHandleStrategyCommand, // Handle strategy commands
    InitIndicatorLookback,          // Initialize indicator lookback
    GetMinIntervalSymbols,          // Get minimum interval symbols
    CalculateIndicator,             // Calculate indicator
    LogNodeState,                   // Log node state
    LogTransition,                  // Log state transition
    LogError(String),               // Log error
    CancelAsyncTask,                // Cancel async task
}

impl StateAction for IndicatorNodeAction {}

// ============================================================================
// IndicatorNode State Transition Function
// ============================================================================

/// IndicatorNode state transition function
///
/// Defines all valid state transitions for IndicatorNode
pub fn indicator_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&strategy_core::node::node_state_machine::Metadata>,
) -> Result<StateChangeActions<NodeRunState, IndicatorNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            NodeRunState::Initializing,
            vec![
                IndicatorNodeAction::LogTransition,
                IndicatorNodeAction::ListenAndHandleExternalEvents,
                IndicatorNodeAction::ListenAndHandleNodeEvents,
                IndicatorNodeAction::ListenAndHandleStrategyCommand,
                IndicatorNodeAction::InitIndicatorLookback,
                IndicatorNodeAction::GetMinIntervalSymbols,
                IndicatorNodeAction::CalculateIndicator,
            ],
        )),

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            NodeRunState::Ready,
            vec![IndicatorNodeAction::LogTransition, IndicatorNodeAction::LogNodeState],
        )),

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![IndicatorNodeAction::LogTransition, IndicatorNodeAction::CancelAsyncTask],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![IndicatorNodeAction::LogTransition, IndicatorNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![IndicatorNodeAction::LogTransition, IndicatorNodeAction::LogError(error.clone())],
        )),

        // Invalid transition
        _ => Err(NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .build()),
    }
}
