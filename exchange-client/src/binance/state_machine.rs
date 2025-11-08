use exchange_core::state_machine::{ExchangeAction, ExchangeRunState, ExchangeStateTransTrigger, ExchangeStateMachine, StateChangeActions, Metadata};
use exchange_core::error::state_machine_error::{ExchangeStateMachineError, ExchangeTransitionSnafu};


pub type BinanceStateMachine = ExchangeStateMachine<BinanceAction>;

#[derive(Debug, Clone)]
pub enum BinanceAction {
    InitHttpClient, // initialize the http client
    InitWsClient,  // initialize the websocket client
    LogExchangeState, // log the state of the binance
    LogTransition, // log the transition result of the binance
    LogError(String), // log the error of the binance
}

impl ExchangeAction for BinanceAction {}

pub fn binance_transition(
    state: &ExchangeRunState,
    trans_trigger: ExchangeStateTransTrigger,
    _metadata: Option<&Metadata>,
) -> Result<StateChangeActions<BinanceAction>, ExchangeStateMachineError> {
    match (state, &trans_trigger) {
        (ExchangeRunState::Created, &ExchangeStateTransTrigger::StartInit) => Ok(StateChangeActions::new(
            ExchangeRunState::Initializing,
            vec![
                BinanceAction::LogTransition,
                BinanceAction::InitHttpClient,
                BinanceAction::InitWsClient,
            ],
        )),

        (ExchangeRunState::Initializing, &ExchangeStateTransTrigger::FinishInit) => Ok(StateChangeActions::new(
            ExchangeRunState::Initialized,
            vec![BinanceAction::LogTransition, BinanceAction::LogExchangeState],
        )),

        (ExchangeRunState::Initialized, &ExchangeStateTransTrigger::Shutdown) => Ok(StateChangeActions::new(
            ExchangeRunState::Stopping,
            vec![BinanceAction::LogTransition],
        )),

        (ExchangeRunState::Stopping, &ExchangeStateTransTrigger::FinishShutdown) => Ok(StateChangeActions::new(
            ExchangeRunState::Stopped,
            vec![BinanceAction::LogTransition, BinanceAction::LogExchangeState],
        )),

        (_, &ExchangeStateTransTrigger::Error(ref error)) => Ok(StateChangeActions::new(
            ExchangeRunState::Error,
            vec![BinanceAction::LogTransition, BinanceAction::LogError(error.clone())],
        )),

        _ => ExchangeTransitionSnafu {
            run_state: state.to_string(),
            trans_trigger: trans_trigger.to_string(),
        }.fail(),
    }
}