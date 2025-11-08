mod strategy_evt_listener;
mod lifecycle;
mod state_handler;
pub(crate) mod strategy_context;
mod strategy_state_machine;
mod strategy_utils;
mod strategy_log_message;

// use super::node;

use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use strategy_core::strategy::StrategyConfig;
use std::sync::Arc;
use strategy_context::BacktestStrategyContext;
use tokio::sync::{Mutex, RwLock};



pub type PlayIndex = i32;






#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}

impl BacktestStrategy {
    pub async fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let context = BacktestStrategyContext::new(strategy_config, database, heartbeat);
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

impl BacktestStrategy {
    pub fn get_context(&self) -> Arc<RwLock<BacktestStrategyContext>> {
        self.context.clone()
    }

    pub async fn with_ctx_read<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a BacktestStrategyContext) -> R + Send,
        R: Send,
    {
        let guard = self.context.read().await;
        f(&*guard)
    }

    pub async fn with_ctx_write<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut BacktestStrategyContext) -> R + Send,
        R: Send,
    {
        let mut guard = self.context.write().await;
        f(&mut *guard)
    }

    pub async fn with_ctx_read_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let guard = self.context.read().await;
        f(&*guard).await
    }

    pub async fn with_ctx_write_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a mut BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let mut guard = self.context.write().await;
        f(&mut *guard).await
    }

    pub async fn read_context(&self) -> tokio::sync::RwLockReadGuard<'_, BacktestStrategyContext> {
        self.context.read().await
    }

    pub async fn write_context(&self) -> tokio::sync::RwLockWriteGuard<'_, BacktestStrategyContext> {
        self.context.write().await
    }
}
