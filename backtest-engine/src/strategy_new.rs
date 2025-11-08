pub(crate) mod strategy_context;
mod strategy_state_machine;
mod strategy_lifecycle;
mod strategy_evt_listener;
mod strategy_log_message;
pub(crate) mod config;

use std::sync::Arc;
use tokio::sync::RwLock;

use strategy_core::strategy::StrategyConfig;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use heartbeat::Heartbeat;
use strategy_context::BacktestStrategyContext;
use strategy_core::strategy::strategy_trait::StrategyContextAccessor;


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
    pub fn new(
        strategy_config: StrategyConfig, 
        database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let context = BacktestStrategyContext::new(strategy_config, database, heartbeat);
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
    
}



impl BacktestStrategy {

}

