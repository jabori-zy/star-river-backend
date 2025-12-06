use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use event_center::{communication::EngineCommand, event::Event};
use star_river_core::engine::EngineName;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use super::context::EngineMetadata;
use crate::{
    state_machine::{EngineAction, EngineRunState, EngineStateMachine, EngineStateTransTrigger, StateChangeActions},
    state_machine_error::EngineStateMachineError,
};

// ============================================================================
// Engine Base Context Trait
// ============================================================================

/// Engine base context trait
///
/// All engine contexts must implement this trait to provide access to base context
///
/// # Associated Types
/// - `Action`: Engine action type, must implement `EngineAction`
#[async_trait]
pub trait EngineContextTrait: Debug + Send + Sync + 'static {
    /// Engine action type
    type Action: EngineAction;

    /// Get immutable reference to base context
    fn base_context(&self) -> &EngineMetadata<Self::Action>;

    /// Get mutable reference to base context
    fn base_context_mut(&mut self) -> &mut EngineMetadata<Self::Action>;

    /// Get engine name
    #[inline]
    fn engine_name(&self) -> &EngineName {
        self.base_context().engine_name()
    }

    /// Get cancellation token
    #[inline]
    fn cancel_token(&self) -> &CancellationToken {
        self.base_context().cancel_token()
    }
}

// ============================================================================
// Extension Trait: EngineEventHandler - Event handling (requires implementation)
// ============================================================================

/// Engine event handler extension
///
/// Defines how engines handle various events, must be implemented by specific engine types
#[async_trait]
pub trait EngineEventHandler: EngineContextTrait {
    /// Handle event
    async fn handle_event(&mut self, event: Event);

    /// Handle command
    async fn handle_command(&mut self, command: EngineCommand);
}

// ============================================================================
// Extension Trait: EngineStateMachineTrait - State machine management
// ============================================================================

/// Engine state machine management extension
///
/// Manages engine runtime state and state transitions
#[async_trait]
pub trait EngineStateMachineTrait: EngineContextTrait {
    /// Get state machine
    fn state_machine(&self) -> Arc<RwLock<EngineStateMachine<Self::Action>>> {
        self.base_context().state_machine()
    }

    /// Get current run state
    #[inline]
    async fn run_state(&self) -> EngineRunState {
        self.state_machine().read().await.current_state().clone()
    }

    /// Check if in specified state
    #[inline]
    async fn is_in_state(&self, state: &EngineRunState) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// State transition
    #[inline]
    async fn transition_state(
        &self,
        trigger: EngineStateTransTrigger,
    ) -> Result<StateChangeActions<Self::Action>, EngineStateMachineError> {
        self.state_machine().write().await.transition(trigger)
    }
}

// Automatically implement EngineStateMachineTrait for all types that implement EngineContextTrait
impl<T> EngineStateMachineTrait for T where T: EngineContextTrait {}
