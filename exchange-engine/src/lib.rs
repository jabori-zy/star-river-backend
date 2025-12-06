mod context;
pub mod error;
mod exchanges;
mod lifecycle;
mod state_machine;

use std::sync::Arc;

use context::ExchangeEngineContext;
use engine_core::{EngineBase, EngineContextAccessor, EngineMetadata, engine_trait::Engine, state_machine::EngineRunState};
use sea_orm::DatabaseConnection;
use star_river_core::engine::EngineName;
use state_machine::ExchangeEngineAction;
use tokio::sync::RwLock;

use crate::{
    error::ExchangeEngineError,
    state_machine::{ExchangeEngineStateMachine, exchange_engine_transition},
};

// ============================================================================
// ExchangeEngine structure (newtype pattern)
// ============================================================================

/// Exchange engine
#[derive(Debug)]
pub struct ExchangeEngine {
    inner: EngineBase<ExchangeEngineContext, ExchangeEngineAction, ExchangeEngineError>,
}

impl ExchangeEngine {
    /// Create a new exchange engine instance
    pub fn new(database: DatabaseConnection) -> Self {
        let state_machine = ExchangeEngineStateMachine::new(
            EngineName::ExchangeEngine.to_string(),
            EngineRunState::Created,
            exchange_engine_transition,
        );
        let base_context = EngineMetadata::new(EngineName::ExchangeEngine, state_machine);
        let context = ExchangeEngineContext::new(base_context, database);

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref implementation - transparent access to inner EngineBase
// ============================================================================

impl std::ops::Deref for ExchangeEngine {
    type Target = EngineBase<ExchangeEngineContext, ExchangeEngineAction, ExchangeEngineError>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for ExchangeEngine {}

// ============================================================================
// EngineContextAccessor implementation - delegated to inner EngineBase
// ============================================================================

impl EngineContextAccessor for ExchangeEngine {
    type Context = ExchangeEngineContext;
    type Action = ExchangeEngineAction;
    type Error = ExchangeEngineError;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}
