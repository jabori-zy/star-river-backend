mod market_engine_context;
mod market_engine_type;
use crate::EngineName;
use crate::{Engine, EngineContext};
use crate::{exchange_engine::ExchangeEngine, market_engine::market_engine_context::MarketEngineContext};
use async_trait::async_trait;
use star_river_core::custom_type::AccountId;
use star_river_core::market::{KlineInterval, Symbol};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct MarketEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}

#[async_trait]
impl Engine for MarketEngine {
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

impl MarketEngine {
    pub fn new(exchange_engine: Arc<Mutex<ExchangeEngine>>) -> Self {
        let context = MarketEngineContext {
            engine_name: EngineName::MarketEngine,
            exchange_engine,
            subscribe_klines: Arc::new(Mutex::new(HashMap::new())),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    pub async fn get_symbol_list(&self, account_id: AccountId) -> Result<Vec<Symbol>, String> {
        let context_read = self.context.read().await;
        let market_engine_context_guard = context_read.as_any().downcast_ref::<MarketEngineContext>().unwrap();
        let symbol_list = market_engine_context_guard.get_symbol_list(account_id).await.unwrap();
        Ok(symbol_list)
    }

    pub async fn get_support_kline_intervals(&self, account_id: AccountId) -> Vec<KlineInterval> {
        let context_read = self.context.read().await;
        let market_engine_context_guard = context_read.as_any().downcast_ref::<MarketEngineContext>().unwrap();
        let support_kline_intervals = market_engine_context_guard.get_support_kline_intervals(account_id).await;
        support_kline_intervals
    }
}
