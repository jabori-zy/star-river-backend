mod event_handler;
mod strategy_manager;
mod strategy_control;

// std
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// third-party
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

// workspace crate
use engine_core::{
    context_trait::EngineContextTrait,
    EngineBaseContext,
};
use star_river_core::custom_type::StrategyId;

// current crate
use crate::{
    state_machine::BacktestEngineAction,
    strategy::BacktestStrategy,
};



#[derive(Debug, Clone)]
pub struct BacktestEngineContext {
    pub base_context: EngineBaseContext<BacktestEngineAction>,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_list: Arc<Mutex<HashMap<StrategyId, BacktestStrategy>>>, // 回测策略列表
    pub initializing_strategies: Arc<Mutex<HashSet<StrategyId>>>,
}


impl BacktestEngineContext {
    pub fn new(
        base_context: EngineBaseContext<BacktestEngineAction>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {

        Self {
            base_context,
            database,
            heartbeat,
            strategy_list: Arc::new(Mutex::new(HashMap::new())),
            initializing_strategies: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}


impl EngineContextTrait for BacktestEngineContext {
    type Action = BacktestEngineAction;

    fn base_context(&self) -> &EngineBaseContext<BacktestEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineBaseContext<BacktestEngineAction> {
        &mut self.base_context
    }

}