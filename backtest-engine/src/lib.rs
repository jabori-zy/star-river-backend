pub mod error;
pub mod backtest_engine_error;
mod context;
mod node_new;
// mod node;
// mod strategy;
pub(crate) mod strategy_new;
// mod node_list;
mod node_list_new;
mod state_machine;
mod engine_lifecycle;
mod node_event;
mod strategy_command;
mod node_command;




// std
use std::sync::Arc;

// third-party
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use tokio::sync::{Mutex, RwLock};

// workspace crate
use engine_core::{
    engine_trait::Engine,
    state_machine::EngineRunState,
    EngineBase, EngineBaseContext, EngineContextAccessor,
};
use star_river_core::engine::EngineName;

// current crate
use crate::{
    context::BacktestEngineContext,
    state_machine::{BacktestEngineAction, BacktestEngineStateMachine, backtest_engine_transition},
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
            backtest_engine_transition
        );
        let base_context = EngineBaseContext::new(
            EngineName::BacktestEngine,
            state_machine
        );


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