pub mod exchange_engine_context;

use crate::EngineName;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use exchange_client::exchange_trait::*;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::AccountId;
use star_river_core::error::engine_error::ExchangeEngineError;
use star_river_core::market::ExchangeStatus;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// 交易所引擎
/// 负责管理交易所客户端，并提供交易所客户端的注册、注销、获取等功能

#[derive(Debug, Clone)]
pub struct ExchangeEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}

#[async_trait]
impl Engine for ExchangeEngine {
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

impl ExchangeEngine {
    pub fn new(database: DatabaseConnection) -> Self {
        let context = ExchangeEngineContext {
            engine_name: EngineName::ExchangeEngine,
            exchanges: HashMap::new(),
            database,
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    pub async fn is_registered(&self, account_id: &i32) -> bool {
        let context_guard = self.context.read().await;
        let exchange_context = context_guard.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();
        exchange_context.is_registered(account_id).await
    }

    pub async fn get_exchange(&self, account_id: &i32) -> Result<Box<dyn ExchangeClientCore>, ExchangeEngineError> {
        let context_guard = self.context.read().await;
        let exchange_context = context_guard.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();
        exchange_context.get_exchange(account_id).await
    }

    pub async fn get_exchange_mut(&self, account_id: &i32) -> Box<dyn ExchangeClientCore> {
        let mut context_guard = self.context.write().await;
        let exchange_context = context_guard.as_any_mut().downcast_mut::<ExchangeEngineContext>().unwrap();
        let client = exchange_context.get_exchange_mut(account_id).await.unwrap();
        client.clone_box()
    }

    pub async fn get_exchange_status(&self, account_id: AccountId) -> ExchangeStatus {
        let context_guard = self.context.read().await;
        let exchange_context = context_guard.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();
        exchange_context.get_exchange_status(&account_id).await
    }

    pub async fn register_exchange(&self, account_id: AccountId) -> Result<(), ExchangeEngineError> {
        let mut context_guard = self.context.write().await;
        let exchange_context = context_guard.as_any_mut().downcast_mut::<ExchangeEngineContext>().unwrap();
        exchange_context.register_exchange(account_id).await
    }
}
