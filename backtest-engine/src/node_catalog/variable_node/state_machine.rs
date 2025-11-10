use strategy_core::{
    error::{NodeStateMachineError, node_state_machine_error::NodeTransFailedSnafu},
    node::node_state_machine::{NodeStateMachine, StateAction, StateChangeActions},
};
use strum::Display;

use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};

// ============================================================================
// VariableNode State Machine Type Alias
// ============================================================================

/// VariableNode state machine type alias
pub type VariableNodeStateMachine = NodeStateMachine<NodeRunState, VariableNodeAction, NodeStateTransTrigger>;

// ============================================================================
// VariableNode Action Definition
// ============================================================================

/// Actions to be executed after VariableNode state transitions
#[derive(Debug, Clone, Display)]
pub enum VariableNodeAction {
    ListenAndHandleNodeEvents,      // Listen and handle node messages
    ListenAndHandleStrategyCommand, // Handle strategy commands
    RegisterTask,                   // Register task
    LogNodeState,                   // Log node state
    LogTransition,                  // Log state transition
    LogError(String),               // Log error
    CancelAsyncTask,                // Cancel async task
}

impl StateAction for VariableNodeAction {}

// ============================================================================
// VariableNode State Transition Function
// ============================================================================

/// VariableNode state transition function
///
/// Defines all valid state transitions for VariableNode
pub fn variable_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&strategy_core::node::node_state_machine::Metadata>,
) -> Result<StateChangeActions<NodeRunState, VariableNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            NodeRunState::Initializing,
            vec![
                VariableNodeAction::LogTransition,
                VariableNodeAction::ListenAndHandleNodeEvents,
                VariableNodeAction::ListenAndHandleStrategyCommand,
            ],
        )),

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            NodeRunState::Ready,
            vec![VariableNodeAction::LogTransition, VariableNodeAction::LogNodeState],
        )),

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![
                VariableNodeAction::LogTransition,
                VariableNodeAction::RegisterTask,
                VariableNodeAction::CancelAsyncTask,
            ],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![VariableNodeAction::LogTransition, VariableNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![VariableNodeAction::LogTransition, VariableNodeAction::LogError(error.clone())],
        )),

        // Invalid transition
        _ => Err(NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .build()),
    }
}
