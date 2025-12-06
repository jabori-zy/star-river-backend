use star_river_core::state_machine::Metadata;
use strategy_core::{
    error::{NodeStateMachineError, node_state_machine_error::NodeTransFailedSnafu},
    node::node_state_machine::{NodeStateMachine, StateAction, StateChangeActions},
};
use strum::Display;

use crate::{
    node::node_state_machine::{NodeRunState, NodeStateTransTrigger},
    strategy::strategy_config::BacktestDataSource,
};
// ============================================================================
// KlineNode Action Definitions
// ============================================================================

/// Actions to execute after KlineNode state transitions
#[derive(Debug, Clone, Display)]
pub enum KlineNodeAction {
    ListenAndHandleExternalEvents,  // Handle external events
    ListenAndHandleNodeEvents,      // Listen to node messages
    ListenAndHandleStrategyCommand, // Handle strategy commands
    LogNodeState,                   // Log node state
    InitMinInterval,                // Initialize minimum interval trading pair
    RegisterExchange,               // Register exchange
    LoadHistoryFromExchange,        // Load kline history from exchange
    LoadHistoryFromFile,            // Load kline history from file
    LogTransition,                  // Log state transition
    LogError(String),               // Log error
    CancelAsyncTask,                // Cancel async task
}

impl StateAction for KlineNodeAction {}

// ============================================================================
// KlineNode State Machine Type Alias
// ============================================================================

/// KlineNode state machine type alias
pub type KlineNodeStateMachine = NodeStateMachine<NodeRunState, KlineNodeAction, NodeStateTransTrigger>;

// ============================================================================
// KlineNode State Transition Function
// ============================================================================

/// KlineNode state transition function
///
/// Determines different initialization flows based on data_source metadata
pub fn kline_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    metadata: Option<&Metadata>,
) -> Result<StateChangeActions<NodeRunState, KlineNodeAction>, NodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => {
            // Read data_source from metadata to determine different initialization actions
            let data_source = metadata
                .and_then(|m| m.get::<BacktestDataSource>("data_source"))
                .unwrap_or(BacktestDataSource::Exchange); // Default to exchange loading

            let actions = match data_source {
                BacktestDataSource::Exchange => vec![
                    KlineNodeAction::LogTransition,
                    KlineNodeAction::LogNodeState,
                    KlineNodeAction::ListenAndHandleExternalEvents,
                    KlineNodeAction::ListenAndHandleNodeEvents,
                    KlineNodeAction::ListenAndHandleStrategyCommand,
                    KlineNodeAction::InitMinInterval,
                    KlineNodeAction::RegisterExchange,
                    KlineNodeAction::LoadHistoryFromExchange,
                ],
                BacktestDataSource::File => vec![
                    KlineNodeAction::LogTransition,
                    KlineNodeAction::LogNodeState,
                    KlineNodeAction::ListenAndHandleExternalEvents,
                    KlineNodeAction::ListenAndHandleNodeEvents,
                    KlineNodeAction::ListenAndHandleStrategyCommand,
                    KlineNodeAction::LoadHistoryFromFile,
                ],
            };

            Ok(StateChangeActions::new(NodeRunState::Initializing, actions))
        }

        // Initializing -> Ready
        (NodeRunState::Initializing, &NodeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            NodeRunState::Ready,
            vec![KlineNodeAction::LogTransition, KlineNodeAction::LogNodeState],
        )),

        // Ready -> Stopping
        (NodeRunState::Ready, &NodeStateTransTrigger::StartStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopping,
            vec![
                KlineNodeAction::LogTransition,
                KlineNodeAction::LogNodeState,
                KlineNodeAction::CancelAsyncTask,
            ],
        )),

        // Stopping -> Stopped
        (NodeRunState::Stopping, &NodeStateTransTrigger::FinishStop) => Ok(StateChangeActions::new(
            NodeRunState::Stopped,
            vec![KlineNodeAction::LogTransition, KlineNodeAction::LogNodeState],
        )),

        // Any state -> Failed
        (_, &NodeStateTransTrigger::EncounterError(ref error)) => Ok(StateChangeActions::new(
            NodeRunState::Failed,
            vec![
                KlineNodeAction::LogTransition,
                KlineNodeAction::LogNodeState,
                KlineNodeAction::LogError(error.clone()),
            ],
        )),

        // Invalid transition
        _ => Err(NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .build()),
    }
}
