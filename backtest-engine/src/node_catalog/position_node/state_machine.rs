use star_river_core::state_machine::Metadata;
use strategy_core::{
    error::{NodeStateMachineError, node_state_machine_error::NodeTransFailedSnafu},
    node::node_state_machine::{NodeStateMachine, StateAction, StateChangeActions},
};
use strum::Display;

use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};
// ============================================================================
// PositionNode State Machine Type Alias
// ============================================================================

/// PositionNode state machine type alias
pub type PositionNodeStateMachine = NodeStateMachine<NodeRunState, PositionNodeAction, NodeStateTransTrigger>;

// ============================================================================
// PositionNode Action Definition
// ============================================================================

/// Actions to be executed after PositionNode state transitions
#[derive(Debug, Clone, Display)]
pub enum PositionNodeAction {
    ListenAndHandleExternalEvents,            // Handle external events (strategy signals)
    ListenAndHandleNodeEvents,                // Listen and handle node messages
    ListenAndHandleStrategyCommand,           // Handle strategy commands
    ListenAndHandleVirtualTradingSystemEvent, // Handle virtual trading system events
    RegisterTask,                             // Register task
    LogNodeState,                             // Log node state
    LogTransition,                            // Log state transition
    LogError(String),                         // Log error
    CancelAsyncTask,                          // Cancel async task
}

impl StateAction for PositionNodeAction {}

// ============================================================================
// PositionNode State Transition Function
// ============================================================================

/// PositionNode state transition function
///
/// Defines all valid state transitions for PositionNode
pub fn position_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<NodeRunState, PositionNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            NodeRunState::Initializing,
            vec![
                PositionNodeAction::LogTransition,
                PositionNodeAction::ListenAndHandleExternalEvents,
                PositionNodeAction::ListenAndHandleNodeEvents,
                PositionNodeAction::ListenAndHandleStrategyCommand,
                PositionNodeAction::ListenAndHandleVirtualTradingSystemEvent,
                PositionNodeAction::RegisterTask,
            ],
        )),

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            NodeRunState::Ready,
            vec![PositionNodeAction::LogTransition, PositionNodeAction::LogNodeState],
        )),

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![PositionNodeAction::LogTransition, PositionNodeAction::CancelAsyncTask],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![PositionNodeAction::LogTransition, PositionNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![PositionNodeAction::LogTransition, PositionNodeAction::LogError(error.clone())],
        )),

        // Invalid transition
        _ => Err(NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .build()),
    }
}
