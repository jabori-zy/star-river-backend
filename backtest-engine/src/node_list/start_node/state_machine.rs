use crate::node::node_state_machine::{NodeRunState, NodeStateMachine, NodeStateTransTrigger, StateChangeActions};
use crate::error::node_state_machine_error::{BacktestNodeStateMachineError, NodeTransFailedSnafu};
use strum::Display;
/// StartNode state machine type alias

pub type StartNodeStateMachine = NodeStateMachine<StartNodeAction>;

// StartNode specific action enum
#[derive(Debug, Clone, Display)]
pub enum StartNodeAction {
    ListenAndHandleExternalEvents,
    ListenAndHandleStrategyCommand,
    ListenAndHandlePlayIndex,
    InitVirtualTradingSystem,
    InitStrategyStats,
    InitCustomVariables,
    LogNodeState,
    LogTransition,
    LogError(String),
    CancelAsyncTask,
}

/// StartNode state transition function
///
/// Defines all valid state transitions for StartNode
pub fn start_node_transition(
    state: &NodeRunState,
    trans_trigger: NodeStateTransTrigger,
    _metadata: Option<&crate::node::node_state_machine::Metadata>,
) -> Result<StateChangeActions<StartNodeAction>, BacktestNodeStateMachineError> {
    match (state, &trans_trigger) {
        // Created -> Initializing
        (NodeRunState::Created, &NodeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            NodeRunState::Initializing,
            vec![
                StartNodeAction::LogTransition,
                StartNodeAction::ListenAndHandleStrategyCommand,
                StartNodeAction::ListenAndHandlePlayIndex,
                StartNodeAction::InitVirtualTradingSystem,
                StartNodeAction::InitStrategyStats,
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
        _ => NodeTransFailedSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
