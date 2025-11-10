use strum::Display;
use strategy_core::node::node_state_machine::{RunState, StateTransTrigger};



#[derive(Debug, Clone, PartialEq, Display)]
pub enum NodeRunState {
    #[strum(to_string = "Checking")]
    Checking,

    #[strum(to_string = "Created")]
    Created,

    #[strum(to_string = "Initializing")]
    Initializing,

    #[strum(to_string = "Ready")]
    Ready,

    #[strum(to_string = "Backtesting")]
    Backtesting,

    #[strum(to_string = "BacktestComplete")]
    BacktestComplete,

    #[strum(to_string = "Stopping")]
    Stopping,

    #[strum(to_string = "Stopped")]
    Stopped,

    #[strum(to_string = "Failed")]
    Failed,
}

impl RunState for NodeRunState {}




#[derive(Debug, Display, Clone)]
pub enum NodeStateTransTrigger {
    #[strum(to_string = "StartInit")]
    StartInit,
    #[strum(to_string = "FinishInit")]
    FinishInit,
    #[strum(to_string = "StartStop")]
    StartStop,
    #[strum(to_string = "FinishStop")]
    FinishStop,
    #[strum(to_string = "EncounterError")]
    EncounterError(String),
}

impl StateTransTrigger for NodeStateTransTrigger {}