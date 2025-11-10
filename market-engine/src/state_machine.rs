use engine_core::{
    state_machine::{EngineAction, EngineRunState, EngineStateMachine, EngineStateTransTrigger, Metadata, StateChangeActions},
    state_machine_error::{EngineStateMachineError, EngineTransitionSnafu},
};

pub type MarketEngineStateMachine = EngineStateMachine<MarketEngineAction>;

#[derive(Debug, Clone)]
pub enum MarketEngineAction {
    ListenAndHandleEvents,
    ListenAndHandleCommands,
    LogEngineState,
    LogTransition,
    LogError(String),
}

impl EngineAction for MarketEngineAction {}

pub fn market_engine_transition(
    state: &EngineRunState,
    trans_trigger: EngineStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<MarketEngineAction>, EngineStateMachineError> {
    match (state, &trans_trigger) {
        (EngineRunState::Created, &EngineStateTransTrigger::Start) => Ok(StateChangeActions::new(
            EngineRunState::Launching,
            vec![
                MarketEngineAction::LogTransition,
                MarketEngineAction::ListenAndHandleEvents,
                MarketEngineAction::ListenAndHandleCommands,
            ],
        )),

        (EngineRunState::Launching, &EngineStateTransTrigger::StartComplete) => Ok(StateChangeActions::new(
            EngineRunState::Running,
            vec![MarketEngineAction::LogTransition, MarketEngineAction::LogEngineState],
        )),

        (EngineRunState::Running, &EngineStateTransTrigger::Stop) => Ok(StateChangeActions::new(
            EngineRunState::Stopping,
            vec![MarketEngineAction::LogTransition],
        )),

        (EngineRunState::Stopping, &EngineStateTransTrigger::StopComplete) => Ok(StateChangeActions::new(
            EngineRunState::Stopped,
            vec![MarketEngineAction::LogTransition, MarketEngineAction::LogEngineState],
        )),

        (_, &EngineStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            EngineRunState::Error,
            vec![MarketEngineAction::LogTransition, MarketEngineAction::LogError(error.clone())],
        )),

        _ => EngineTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
