mod engine_context;
mod log_message;
mod node;
mod strategy;
pub mod strategy_data_query;
pub mod strategy_control;

use strategy::strategy_context::BacktestStrategyContext;
use crate::EngineName;
use crate::{Engine, EngineContext};
use crate::backtest_strategy_engine::engine_context::StrategyEngineContext;
use async_trait::async_trait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::error::engine_error::*;
use star_river_core::strategy::TradeMode;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct BacktestStrategyEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}

#[async_trait]
impl Engine for BacktestStrategyEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>> {
        self.context.clone()
    }
}

impl BacktestStrategyEngine {
    pub fn new(database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let context = StrategyEngineContext {
            engine_name: EngineName::StrategyEngine,
            database,
            strategy_list: Arc::new(Mutex::new(HashMap::new())),
            initializing_strategies: Arc::new(Mutex::new(HashSet::new())),
            heartbeat,
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }


    async fn get_strategy_context(&self, strategy_id: i32)
          -> Result<Arc<RwLock<BacktestStrategyContext>>, StrategyEngineError> {
          let context = self.context.read().await;
          let strategy_context: &StrategyEngineContext = context.as_any()
              .downcast_ref::<StrategyEngineContext>()
              .unwrap();
          let strategy = strategy_context.get_strategy_instance(strategy_id).await?;
          Ok(strategy.get_context())
          }


    
}

