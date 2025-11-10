// Workspace crate imports
use engine_core::{
    state_machine::{EngineAction, EngineRunState, EngineStateMachine, EngineStateTransTrigger, Metadata, StateChangeActions},
    state_machine_error::{EngineStateMachineError, EngineTransitionSnafu},
};

pub type BacktestEngineStateMachine = EngineStateMachine<BacktestEngineAction>;

#[derive(Debug, Clone)]
pub enum BacktestEngineAction {
    ListenAndHandleEvents,
    ListenAndHandleCommands,
    LogEngineState,
    LogTransition,
    LogError(String),
}

impl EngineAction for BacktestEngineAction {}

pub fn backtest_engine_transition(
    state: &EngineRunState,
    trans_trigger: EngineStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<BacktestEngineAction>, EngineStateMachineError> {
    match (state, &trans_trigger) {
        (EngineRunState::Created, &EngineStateTransTrigger::Start) => Ok(StateChangeActions::new(
            EngineRunState::Launching,
            vec![
                BacktestEngineAction::LogTransition,
                BacktestEngineAction::ListenAndHandleEvents,
                BacktestEngineAction::ListenAndHandleCommands,
            ],
        )),

        (EngineRunState::Launching, &EngineStateTransTrigger::StartComplete) => Ok(StateChangeActions::new(
            EngineRunState::Running,
            vec![BacktestEngineAction::LogTransition, BacktestEngineAction::LogEngineState],
        )),

        (EngineRunState::Running, &EngineStateTransTrigger::Stop) => Ok(StateChangeActions::new(
            EngineRunState::Stopping,
            vec![BacktestEngineAction::LogTransition],
        )),

        (EngineRunState::Stopping, &EngineStateTransTrigger::StopComplete) => Ok(StateChangeActions::new(
            EngineRunState::Stopped,
            vec![BacktestEngineAction::LogTransition, BacktestEngineAction::LogEngineState],
        )),

        (_, &EngineStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            EngineRunState::Error,
            vec![BacktestEngineAction::LogTransition, BacktestEngineAction::LogError(error.clone())],
        )),

        _ => EngineTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
