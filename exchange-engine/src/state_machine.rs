use engine_core::{
    state_machine::{EngineAction, EngineRunState, EngineStateMachine, EngineStateTransTrigger, Metadata, StateChangeActions},
    state_machine_error::{EngineStateMachineError, EngineTransitionSnafu},
};

pub type ExchangeEngineStateMachine = EngineStateMachine<ExchangeEngineAction>;

#[derive(Debug, Clone)]
pub enum ExchangeEngineAction {
    ListenAndHandleEvents,
    ListenAndHandleCommands,
    LogEngineState,
    LogTransition,
    LogError(String),
}

impl EngineAction for ExchangeEngineAction {}

pub fn exchange_engine_transition(
    state: &EngineRunState,
    trans_trigger: EngineStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<ExchangeEngineAction>, EngineStateMachineError> {
    match (state, &trans_trigger) {
        (EngineRunState::Created, &EngineStateTransTrigger::Start) => Ok(StateChangeActions::new(
            EngineRunState::Launching,
            vec![
                ExchangeEngineAction::LogTransition,
                ExchangeEngineAction::ListenAndHandleEvents,
                ExchangeEngineAction::ListenAndHandleCommands,
            ],
        )),

        (EngineRunState::Launching, &EngineStateTransTrigger::StartComplete) => Ok(StateChangeActions::new(
            EngineRunState::Running,
            vec![ExchangeEngineAction::LogTransition, ExchangeEngineAction::LogEngineState],
        )),

        (EngineRunState::Running, &EngineStateTransTrigger::Stop) => Ok(StateChangeActions::new(
            EngineRunState::Stopping,
            vec![ExchangeEngineAction::LogTransition],
        )),

        (EngineRunState::Stopping, &EngineStateTransTrigger::StopComplete) => Ok(StateChangeActions::new(
            EngineRunState::Stopped,
            vec![ExchangeEngineAction::LogTransition, ExchangeEngineAction::LogEngineState],
        )),

        (_, &EngineStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            EngineRunState::Error,
            vec![ExchangeEngineAction::LogTransition, ExchangeEngineAction::LogError(error.clone())],
        )),

        _ => EngineTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
