mod context;
pub mod engine_error;
mod engine_lifecycle;
mod engine_state_machine;
mod node;
mod node_catalog;
pub(crate) mod strategy;

// Standard library imports
use std::sync::Arc;

// Workspace crate imports
use engine_core::{EngineBase, EngineBaseContext, EngineContextAccessor, engine_trait::Engine, state_machine::EngineRunState};
// External crate imports
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::engine::EngineName;
use tokio::sync::{Mutex, RwLock};

// Current crate imports
use crate::{
    context::BacktestEngineContext,
    engine_state_machine::{BacktestEngineAction, BacktestEngineStateMachine, backtest_engine_transition},
};

/// 回测引擎
#[derive(Debug)]
pub struct BacktestEngine {
    inner: EngineBase<BacktestEngineContext, BacktestEngineAction>,
}

impl BacktestEngine {
    pub fn new(database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let state_machine = BacktestEngineStateMachine::new(
            EngineName::BacktestEngine.to_string(),
            EngineRunState::Created,
            backtest_engine_transition,
        );
        let base_context = EngineBaseContext::new(EngineName::BacktestEngine, state_machine);

        let context = BacktestEngineContext::new(base_context, database, heartbeat);
        Self {
            inner: EngineBase::new(context),
        }
    }
}

impl std::ops::Deref for BacktestEngine {
    type Target = EngineBase<BacktestEngineContext, BacktestEngineAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for BacktestEngine {}

impl EngineContextAccessor for BacktestEngine {
    type Context = BacktestEngineContext;
    type Action = BacktestEngineAction;
    fn context(&self) -> &Arc<RwLock<BacktestEngineContext>> {
        self.inner.context()
    }
}
