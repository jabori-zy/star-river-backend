use exchange_core::{
    error::{ExchangeStateMachineError, state_machine_error::ExchangeTransitionSnafu},
    state_machine::{ExchangeAction, ExchangeRunState, ExchangeStateMachine, ExchangeStateTransTrigger, Metadata, StateChangeActions},
};

pub type Mt5StateMachine = ExchangeStateMachine<Mt5Action>;

#[derive(Debug, Clone)]
pub enum Mt5Action {
    InitHttpClient,   // initialize the http client
    InitWsClient,     // initialize the websocket client
    LogExchangeState, // log the state of the metatrader5
    LogTransition,    // log the transition result of the metatrader5
    LogError(String), // log the error of the metatrader5
}

impl ExchangeAction for Mt5Action {}

pub fn metatrader5_transition(
    state: &ExchangeRunState,
    trans_trigger: ExchangeStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<Mt5Action>, ExchangeStateMachineError> {
    match (state, &trans_trigger) {
        (ExchangeRunState::Created, &ExchangeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            ExchangeRunState::Initializing,
            vec![Mt5Action::LogTransition, Mt5Action::InitHttpClient, Mt5Action::InitWsClient],
        )),

        (ExchangeRunState::Initializing, &ExchangeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            ExchangeRunState::Connected,
            vec![Mt5Action::LogTransition, Mt5Action::LogExchangeState],
        )),

        (ExchangeRunState::Connected, &ExchangeStateTransTrigger::Shutdown) => {
            Ok(StateChangeActions::new(ExchangeRunState::Stopping, vec![Mt5Action::LogTransition]))
        }

        (ExchangeRunState::Stopping, &ExchangeStateTransTrigger::FinishShutdown) => Ok(StateChangeActions::new(
            ExchangeRunState::Stopped,
            vec![Mt5Action::LogTransition, Mt5Action::LogExchangeState],
        )),

        (_, &ExchangeStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            ExchangeRunState::Error,
            vec![Mt5Action::LogTransition, Mt5Action::LogError(error.clone())],
        )),

        _ => ExchangeTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }
        .fail(),
    }
}
