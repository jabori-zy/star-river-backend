use engine_core::{
    state_machine::{EngineAction, EngineRunState, EngineStateMachine, EngineStateTransTrigger, Metadata, StateChangeActions},
    state_machine_error::{EngineStateMachineError, EngineTransitionSnafu},
};

pub type IndicatorEngineStateMachine = EngineStateMachine<IndicatorEngineAction>;

#[derive(Debug, Clone)]
pub enum IndicatorEngineAction {
    ListenAndHandleEvents,
    ListenAndHandleCommands,
    LogEngineState,
    LogTransition,
    LogError(String),
}

impl EngineAction for IndicatorEngineAction {}

pub fn indicator_engine_transition(
    state: &EngineRunState,
    trans_trigger: EngineStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<IndicatorEngineAction>, EngineStateMachineError> {
    match (state, &trans_trigger) {
        (EngineRunState::Created, &EngineStateTransTrigger::Start) => Ok(StateChangeActions::new(
            EngineRunState::Launching,
            vec![
                IndicatorEngineAction::LogTransition,
                IndicatorEngineAction::ListenAndHandleEvents,
                IndicatorEngineAction::ListenAndHandleCommands,
            ],
        )),

        (EngineRunState::Launching, &EngineStateTransTrigger::StartComplete) => Ok(StateChangeActions::new(
            EngineRunState::Running,
            vec![IndicatorEngineAction::LogTransition, IndicatorEngineAction::LogEngineState],
        )),

        (EngineRunState::Running, &EngineStateTransTrigger::Stop) => Ok(StateChangeActions::new(
            EngineRunState::Stopping,
            vec![IndicatorEngineAction::LogTransition],
        )),

        (EngineRunState::Stopping, &EngineStateTransTrigger::StopComplete) => Ok(StateChangeActions::new(
            EngineRunState::Stopped,
            vec![IndicatorEngineAction::LogTransition, IndicatorEngineAction::LogEngineState],
        )),

        (_, &EngineStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            EngineRunState::Error,
            vec![IndicatorEngineAction::LogTransition, IndicatorEngineAction::LogError(error.clone())],
        )),

        _ => EngineTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
