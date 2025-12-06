use star_river_core::state_machine::Metadata;
use strategy_core::{
    error::{NodeStateMachineError, node_state_machine_error::NodeTransFailedSnafu},
    node::node_state_machine::{NodeStateMachine, StateAction, StateChangeActions},
};
use strum::Display;

use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};
/// StartNode state machine type alias
pub type StartNodeStateMachine = NodeStateMachine<NodeRunState, StartNodeAction, NodeStateTransTrigger>;

// ============================================================================
// StartNode Action Definitions
// ============================================================================

/// StartNode specific action enum
#[derive(Debug, Clone, Display)]
pub enum StartNodeAction {
    ListenAndHandleExternalEvents,
    ListenAndHandleStrategyCommand,
    ListenAndHandlePlayIndex,
    // InitVirtualTradingSystem,
    // InitStrategyStats,
    InitCustomVariables,
    LogNodeState,
    LogTransition,
    LogError(String),
    CancelAsyncTask,
}

impl StateAction for StartNodeAction {}

// ============================================================================
// StartNode State Machine Type Alias
// ============================================================================

// ============================================================================
// StartNode State Transition Function
// ============================================================================

/// StartNode state transition function
///
/// Defines all valid state transitions for StartNode
pub fn start_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<NodeRunState, StartNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            NodeRunState::Initializing,
            vec![
                StartNodeAction::LogTransition,
                StartNodeAction::ListenAndHandleStrategyCommand,
                StartNodeAction::ListenAndHandlePlayIndex,
                // StartNodeAction::InitVirtualTradingSystem,
                // StartNodeAction::InitStrategyStats,
                StartNodeAction::InitCustomVariables,
            ],
        )),

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            NodeRunState::Ready,
            vec![StartNodeAction::LogTransition, StartNodeAction::LogNodeState],
        )),

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![StartNodeAction::LogTransition, StartNodeAction::CancelAsyncTask],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![StartNodeAction::LogTransition, StartNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![StartNodeAction::LogTransition, StartNodeAction::LogError(error.clone())],
        )),

        // Invalid transition
        _ => Err(NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .build()),
    }
}
