// ============================================================================
// Backtest Strategy State Machine - Generic Implementation
// ============================================================================
//
// This module implements a generic backtest strategy state machine based on
// the strategy-core generic state machine framework. It references the original
// BacktestStrategyStateMachine design while providing better type safety and extensibility.

use star_river_core::{custom_type::StrategyName, state_machine::Metadata};
use strategy_core::{
    error::{StrategyStateMachineError, strategy_state_machine_error::StrategyStateTransFailedSnafu},
    strategy::state_machine::{
        GenericStrategyStateMachine, StrategyRunState, StrategyStateAction, StrategyStateChangeActions, StrategyStateMachine,
        StrategyStateTransTrigger,
    },
};
use strum::Display;

// ============================================================================
// State Definitions
// ============================================================================

/// Backtest strategy run states
#[derive(Debug, Clone, PartialEq, Display)]
pub enum BacktestStrategyRunState {
    /// Strategy created but not initialized
    #[strum(serialize = "Created")]
    Created,

    /// Strategy is checking configuration
    #[strum(serialize = "Checking")]
    Checking,

    /// Strategy check passed
    #[strum(serialize = "CheckPassed")]
    CheckPassed,

    /// Strategy is initializing
    #[strum(serialize = "Initializing")]
    Initializing,

    /// Strategy is ready
    #[strum(serialize = "Ready")]
    Ready,

    /// Strategy is playing/backtesting
    #[strum(serialize = "Playing")]
    Playing,

    /// Strategy is pausing
    #[strum(serialize = "Pausing")]
    Pausing,

    /// Strategy playback completed
    #[strum(serialize = "PlayComplete")]
    PlayComplete,

    /// Strategy is stopping
    #[strum(serialize = "Stopping")]
    Stopping,

    /// Strategy stopped
    #[strum(serialize = "Stopped")]
    Stopped,

    /// Strategy failed
    #[strum(serialize = "Failed")]
    Failed,
}

impl StrategyRunState for BacktestStrategyRunState {}

// ============================================================================
// State Transition Trigger Definitions
// ============================================================================

/// Strategy state transition triggers
#[derive(Debug, Clone, Display)]
pub enum BacktestStrategyStateTransTrigger {
    /// Start checking strategy
    #[strum(serialize = "Check")]
    Check,

    /// Check completed
    #[strum(serialize = "CheckComplete")]
    CheckComplete,

    /// Start initialization
    #[strum(serialize = "Initialize")]
    Initialize,

    /// Initialization completed
    #[strum(serialize = "InitializeComplete")]
    InitializeComplete,

    /// Stop strategy
    #[strum(serialize = "Stop")]
    Stop,

    /// Stop completed
    #[strum(serialize = "StopComplete")]
    StopComplete,

    /// Strategy failed
    #[strum(serialize = "Fail")]
    Fail(String),
}

impl StrategyStateTransTrigger for BacktestStrategyStateTransTrigger {}

// ============================================================================
// State Action Definitions
// ============================================================================

/// Actions to execute during backtest strategy state transitions
#[derive(Debug, Clone, Display)]
pub enum BacktestStrategyStateAction {
    /// Initialize signal count
    #[strum(serialize = "InitSignalCount")]
    InitSignalCount,

    /// Initialize initial playback speed
    #[strum(serialize = "InitInitialPlaySpeed")]
    InitInitialPlaySpeed,

    /// Initialize virtual trading system
    #[strum(serialize = "InitVirtualTradingSystem")]
    InitVirtualTradingSystem,

    /// Initialize strategy statistics
    #[strum(serialize = "InitStrategyStats")]
    InitStrategyStats,

    /// Check nodes
    #[strum(serialize = "CheckNode")]
    CheckNode,

    /// Initialize nodes
    #[strum(serialize = "InitNode")]
    InitNode,

    /// Stop nodes
    #[strum(serialize = "StopNode")]
    StopNode,

    /// Listen and handle node events
    #[strum(serialize = "ListenAndHandleNodeEvent")]
    ListenAndHandleNodeEvent,

    /// Listen and handle strategy commands
    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand,

    /// Listen and handle strategy statistics events
    #[strum(serialize = "ListenAndHandleStrategyStatsEvent")]
    ListenAndHandleStrategyStatsEvent,

    /// Log strategy state
    #[strum(serialize = "LogStrategyState")]
    LogStrategyState,

    /// Log state transition
    #[strum(serialize = "LogTransition")]
    LogTransition,

    /// Log error
    #[strum(serialize = "LogError")]
    LogError(String),
}

impl StrategyStateAction for BacktestStrategyStateAction {}

// ============================================================================
// State Transition Function
// ============================================================================

/// State transition function - implements specific state transition logic
///
/// # Arguments
/// - `current_state`: Current state
/// - `trigger`: Event that triggers the state transition
/// - `metadata`: Optional metadata
///
/// # Returns
/// - `Ok(StrategyStateChangeActions)`: Transition successful, contains new state and action list
/// - `Err(StrategyStateMachineError)`: Transition failed
pub fn backtest_strategy_transition(
    current_state: &BacktestStrategyRunState,
    trigger: BacktestStrategyStateTransTrigger,
    strategy_name: StrategyName,
    _metadata: Option<&Metadata>,
) -> Result<StrategyStateChangeActions<BacktestStrategyRunState, BacktestStrategyStateAction>, StrategyStateMachineError> {
    match (current_state, trigger) {
        // Created -> Checking: Start checking strategy
        (BacktestStrategyRunState::Created, BacktestStrategyStateTransTrigger::Check) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::Checking,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
                BacktestStrategyStateAction::CheckNode,
            ],
        )),

        // Checking -> CheckPassed: Check completed
        (BacktestStrategyRunState::Checking, BacktestStrategyStateTransTrigger::CheckComplete) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::CheckPassed,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
            ],
        )),

        // CheckPassed -> Initializing: Start initialization
        (BacktestStrategyRunState::CheckPassed, BacktestStrategyStateTransTrigger::Initialize) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::Initializing,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
                BacktestStrategyStateAction::ListenAndHandleNodeEvent,
                BacktestStrategyStateAction::ListenAndHandleStrategyCommand,
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent,
                BacktestStrategyStateAction::InitNode,
                BacktestStrategyStateAction::InitSignalCount,
                BacktestStrategyStateAction::InitInitialPlaySpeed,
                BacktestStrategyStateAction::InitVirtualTradingSystem,
                BacktestStrategyStateAction::InitStrategyStats,
            ],
        )),

        // Initializing -> Ready: Initialization completed
        (BacktestStrategyRunState::Initializing, BacktestStrategyStateTransTrigger::InitializeComplete) => {
            Ok(StrategyStateChangeActions::new(
                BacktestStrategyRunState::Ready,
                vec![
                    BacktestStrategyStateAction::LogTransition,
                    BacktestStrategyStateAction::LogStrategyState,
                ],
            ))
        }

        // Ready -> Stopping: Stop strategy
        (BacktestStrategyRunState::Ready, BacktestStrategyStateTransTrigger::Stop) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::Stopping,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
                BacktestStrategyStateAction::StopNode,
            ],
        )),

        // Stopping -> Stopped: Stop completed
        (BacktestStrategyRunState::Stopping, BacktestStrategyStateTransTrigger::StopComplete) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::Stopped,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
            ],
        )),

        // Any state -> Failed: Strategy failed
        (_, BacktestStrategyStateTransTrigger::Fail(error)) => Ok(StrategyStateChangeActions::new(
            BacktestStrategyRunState::Failed,
            vec![
                BacktestStrategyStateAction::LogTransition,
                BacktestStrategyStateAction::LogStrategyState,
                BacktestStrategyStateAction::LogError(error),
            ],
        )),

        // Invalid state transition
        (state, trigger) => Err(StrategyStateTransFailedSnafu {
            strategy_name: strategy_name.clone(),
            run_state: state.to_string(),
            trans_trigger: trigger.to_string(),
        }
        .build()),
    }
}

// ============================================================================
// Type Alias
// ============================================================================

/// Backtest strategy state machine type alias
pub type BacktestStrategyStateMachine =
    GenericStrategyStateMachine<BacktestStrategyRunState, BacktestStrategyStateAction, BacktestStrategyStateTransTrigger>;
