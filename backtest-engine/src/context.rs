mod event_handler;
mod strategy_control;
mod strategy_manager;

// Standard library imports
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// Workspace crate imports
use engine_core::{EngineMetadata, context_trait::EngineContextTrait};
// External crate imports
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::StrategyId;
use strategy_core::strategy::strategy_trait::StrategyContextAccessor;
use tokio::sync::Mutex;

// Current crate imports
use crate::{
    engine_state_machine::BacktestEngineAction,
    strategy::{BacktestStrategy, strategy_context::BacktestStrategyContext},
};

#[derive(Debug, Clone)]
pub struct BacktestEngineContext {
    pub base_context: EngineMetadata<BacktestEngineAction>,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_list: Arc<Mutex<HashMap<StrategyId, BacktestStrategy>>>, // Backtest strategy list
    pub initializing_strategies: Arc<Mutex<HashSet<StrategyId>>>,
}

impl BacktestEngineContext {
    pub fn new(base_context: EngineMetadata<BacktestEngineAction>, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        Self {
            base_context,
            database,
            heartbeat,
            strategy_list: Arc::new(Mutex::new(HashMap::new())),
            initializing_strategies: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn with_strategy<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a BacktestStrategy) -> R + Send,
        R: Send,
    {
        use snafu::OptionExt;

        use crate::engine_error::StrategyInstanceNotFoundSnafu;

        let guard = self.strategy_list.lock().await;
        let strategy = guard.get(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy))
    }

    pub async fn with_strategy_mut<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a mut BacktestStrategy) -> R + Send,
        R: Send,
    {
        use snafu::OptionExt;

        use crate::engine_error::StrategyInstanceNotFoundSnafu;

        let mut guard = self.strategy_list.lock().await;
        let strategy = guard.get_mut(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy))
    }

    pub async fn with_strategy_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a BacktestStrategy) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send,
    ) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        R: Send,
    {
        use snafu::OptionExt;

        use crate::engine_error::StrategyInstanceNotFoundSnafu;

        let guard = self.strategy_list.lock().await;
        let strategy = guard.get(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy).await)
    }

    pub async fn with_strategy_mut_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a mut BacktestStrategy) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send,
    ) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        R: Send,
    {
        use snafu::OptionExt;

        use crate::engine_error::StrategyInstanceNotFoundSnafu;

        let mut guard = self.strategy_list.lock().await;
        let strategy = guard.get_mut(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy).await)
    }

    /// Access strategy context with read lock (sync closure)
    ///
    /// This is a helper method to directly access strategy context, reducing nesting levels
    ///
    /// # Example
    /// ```rust
    /// context.with_strategy_ctx_read(strategy_id, |ctx| {
    ///     ctx.cycle_id()
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_read<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a BacktestStrategyContext) -> R + Send + 'static,
        R: Send,
    {
        self.with_strategy_async(strategy_id, |strategy| Box::pin(async move { strategy.with_ctx_read(f).await }))
            .await
    }

    /// Access strategy context with read lock (async closure)
    ///
    /// This is a helper method to directly access strategy context, reducing nesting levels
    ///
    /// # Example
    /// ```rust
    /// context.with_strategy_ctx_read_async(strategy_id, |ctx| {
    ///     Box::pin(async move {
    ///         ctx.get_virtual_orders().await
    ///     })
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_read_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>>
        + Send
        + 'static,
    ) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        R: Send,
    {
        self.with_strategy_async(strategy_id, |strategy| {
            Box::pin(async move { strategy.with_ctx_read_async(f).await })
        })
        .await
    }

    /// Access strategy context with write lock (sync closure)
    ///
    /// This is a helper method to directly access strategy context, reducing nesting levels
    ///
    /// # Example
    /// ```rust
    /// context.with_strategy_ctx_write(strategy_id, |ctx| {
    ///     ctx.set_something(value)
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_write<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a mut BacktestStrategyContext) -> R + Send + 'static,
        R: Send,
    {
        self.with_strategy_mut_async(strategy_id, |strategy| Box::pin(async move { strategy.with_ctx_write(f).await }))
            .await
    }

    /// Access strategy context with write lock (async closure)
    ///
    /// This is a helper method to directly access strategy context, reducing nesting levels
    ///
    /// # Example
    /// ```rust
    /// context.with_strategy_ctx_write_async(strategy_id, |ctx| {
    ///     Box::pin(async move {
    ///         ctx.update_something().await
    ///     })
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_write_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a mut BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>>
        + Send
        + 'static,
    ) -> Result<R, crate::engine_error::BacktestEngineError>
    where
        R: Send,
    {
        self.with_strategy_mut_async(strategy_id, |strategy| {
            Box::pin(async move { strategy.with_ctx_write_async(f).await })
        })
        .await
    }
}

impl EngineContextTrait for BacktestEngineContext {
    type Action = BacktestEngineAction;

    fn base_context(&self) -> &EngineMetadata<BacktestEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineMetadata<BacktestEngineAction> {
        &mut self.base_context
    }
}
