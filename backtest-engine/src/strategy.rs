pub(crate) mod strategy_command;
pub(crate) mod strategy_config;
pub(crate) mod strategy_context;
pub(crate) mod strategy_error;
mod strategy_evt_listener;
mod strategy_lifecycle;
mod strategy_log_message;
pub(crate) mod strategy_state_machine;

// Standard library imports
use std::sync::Arc;

// External crate imports
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
// Current crate imports
use strategy_context::BacktestStrategyContext;
// Workspace crate imports
use strategy_core::strategy::{StrategyConfig, strategy_trait::StrategyContextAccessor};
use tokio::sync::{Mutex, RwLock};

pub type PlayIndex = i32;

#[derive(Debug)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}

impl StrategyContextAccessor for BacktestStrategy {
    type Context = BacktestStrategyContext;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}

impl BacktestStrategy {
    pub fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let context = BacktestStrategyContext::new(strategy_config, database, heartbeat);
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

impl BacktestStrategy {}
