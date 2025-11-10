use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};
use strategy_core::error::node_state_machine_error::NodeTransFailedSnafu;
use strategy_core::node::node_state_machine::{NodeStateMachine, StateChangeActions, StateAction};
use strategy_core::error::NodeStateMachineError;
use strum::Display;


// ============================================================================
// IfElseNode State Machine Type Alias
// ============================================================================

/// IfElseNode state machine type alias
pub type IfElseNodeStateMachine = NodeStateMachine<NodeRunState, IfElseNodeAction, NodeStateTransTrigger>;

// ============================================================================
// IfElseNode Action Definition
// ============================================================================

/// Actions to be executed after IfElseNode state transitions
#[derive(Debug, Clone, Display)]
pub enum IfElseNodeAction {
    ListenAndHandleStrategySignal,  // Handle external events (strategy signals)
    ListenAndHandleNodeEvents,      // Listen and handle node messages
    ListenAndHandleStrategyCommand, // Handle strategy commands
    InitReceivedData,               // Initialize received data storage
    Evaluate,                       // Evaluate condition and route to appropriate branch
    LogNodeState,                   // Log node state
    LogTransition,                  // Log state transition
    LogError(String),               // Log error
    CancelAsyncTask,                // Cancel async task
}

impl StateAction for IfElseNodeAction {}



// ============================================================================
// IfElseNode State Transition Function
// ============================================================================

/// IfElseNode state transition function
///
/// Defines all valid state transitions for IfElseNode
pub fn if_else_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&strategy_core::node::node_state_machine::Metadata>,
) -> Result<StateChangeActions<NodeRunState, IfElseNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => {
            Ok(StateChangeActions::new(
                NodeRunState::Initializing,
                vec![
                    IfElseNodeAction::LogTransition,
                    IfElseNodeAction::ListenAndHandleStrategySignal,
                    IfElseNodeAction::ListenAndHandleNodeEvents,
                    IfElseNodeAction::ListenAndHandleStrategyCommand,
                    IfElseNodeAction::InitReceivedData,
                ],
            ))
        }

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => {
            Ok(StateChangeActions::new(
                NodeRunState::Ready,
                vec![
                    IfElseNodeAction::LogTransition,
                    IfElseNodeAction::LogNodeState,
                    IfElseNodeAction::Evaluate,
                ],
            ))
        }

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => {
            Ok(StateChangeActions::new(
                NodeRunState::Stopping,
                vec![
                    IfElseNodeAction::LogTransition,
                    IfElseNodeAction::CancelAsyncTask,
                ],
            ))
        }

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => {
            Ok(StateChangeActions::new(
                NodeRunState::Stopped,
                vec![
                    IfElseNodeAction::LogTransition,
                    IfElseNodeAction::LogNodeState,
                ],
            ))
        }

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => {
            Ok(StateChangeActions::new(
                NodeRunState::Failed,
                vec![
                    IfElseNodeAction::LogTransition,
                    IfElseNodeAction::LogError(error.clone()),
                ],
            ))
        }

        // Invalid transition
        _ => Err(NodeTransFailedSnafu{
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }.build()),
    }
}
