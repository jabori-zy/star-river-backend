use crate::node::node_state_machine::{NodeRunState, NodeStateTransTrigger};
use strategy_core::error::node_state_machine_error::NodeTransFailedSnafu;
use strategy_core::node::node_state_machine::{NodeStateMachine, StateChangeActions, StateAction};
use strategy_core::error::NodeStateMachineError;
use strum::Display;


// ============================================================================
// FuturesOrderNode State Machine Type Alias
// ============================================================================

/// FuturesOrderNode state machine type alias
pub type FuturesOrderNodeStateMachine = NodeStateMachine<NodeRunState, FuturesOrderNodeAction, NodeStateTransTrigger>;

// ============================================================================
// FuturesOrderNode Action Definition
// ============================================================================

/// Actions to be executed after FuturesOrderNode state transitions
#[derive(Debug, Clone, Display)]
pub enum FuturesOrderNodeAction {
    ListenAndHandleExternalEvents,              // Handle external events (strategy signals)
    ListenAndHandleNodeEvents,                  // Listen and handle node messages
    ListenAndHandleStrategyCommand,             // Handle strategy commands
    ListenAndHandleVirtualTradingSystemEvent,   // Handle virtual trading system events
    GetSymbolInfo,                              // Get trading pair information
    RegisterTask,                               // Register task
    LogNodeState,                               // Log node state
    LogTransition,                              // Log state transition
    LogError(String),                           // Log error
    CancelAsyncTask,                            // Cancel async task
}

impl StateAction for FuturesOrderNodeAction {}



// ============================================================================
// FuturesOrderNode State Transition Function
// ============================================================================

/// FuturesOrderNode state transition function
///
/// Defines all valid state transitions for FuturesOrderNode
pub fn futures_order_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&strategy_core::node::node_state_machine::Metadata>,
) -> Result<StateChangeActions<NodeRunState, FuturesOrderNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => {
            Ok(StateChangeActions::new(
                NodeRunState::Initializing,
                vec![
                    FuturesOrderNodeAction::LogTransition,
                    FuturesOrderNodeAction::ListenAndHandleExternalEvents,
                    FuturesOrderNodeAction::ListenAndHandleNodeEvents,
                    FuturesOrderNodeAction::ListenAndHandleStrategyCommand,
                    FuturesOrderNodeAction::ListenAndHandleVirtualTradingSystemEvent,
                    FuturesOrderNodeAction::GetSymbolInfo,
                    FuturesOrderNodeAction::RegisterTask,
                ],
            ))
        }

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => {
            Ok(StateChangeActions::new(
                NodeRunState::Ready,
                vec![
                    FuturesOrderNodeAction::LogTransition,
                    FuturesOrderNodeAction::LogNodeState,
                ],
            ))
        }

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => {
            Ok(StateChangeActions::new(
                NodeRunState::Stopping,
                vec![
                    FuturesOrderNodeAction::LogTransition,
                    FuturesOrderNodeAction::CancelAsyncTask,
                ],
            ))
        }

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => {
            Ok(StateChangeActions::new(
                NodeRunState::Stopped,
                vec![
                    FuturesOrderNodeAction::LogTransition,
                    FuturesOrderNodeAction::LogNodeState,
                ],
            ))
        }

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => {
            Ok(StateChangeActions::new(
                NodeRunState::Failed,
                vec![
                    FuturesOrderNodeAction::LogTransition,
                    FuturesOrderNodeAction::LogError(error.clone()),
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
