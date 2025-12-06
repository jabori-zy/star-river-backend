use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::context_trait::StrategyStatsMetaDataExt;

#[async_trait]
pub trait StrategyStatsAccessor: Send + Sync {
    /// Context type, must implement StrategyMetaDataExt
    type Context: StrategyStatsMetaDataExt;

    /// Get shared reference to context
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// Access context with read lock (sync closure)
    ///
    /// # Example
    /// ```rust
    /// let strategy_name = strategy.with_ctx_read(|ctx| {
    ///     ctx.strategy_name().to_string()
    /// }).await;
    /// ```
    async fn with_ctx_read<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a Self::Context) -> R + Send,
        R: Send,
    {
        let guard = self.context().read().await;
        f(&*guard)
    }

    /// Access context with write lock (sync closure)
    ///
    /// # Example
    /// ```rust
    /// strategy.with_ctx_write(|ctx| {
    ///     // Modify context
    /// }).await;
    /// ```
    async fn with_ctx_write<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut Self::Context) -> R + Send,
        R: Send,
    {
        let mut guard = self.context().write().await;
        f(&mut *guard)
    }

    /// Access context with read lock (async closure)
    ///
    /// # Example
    /// ```rust
    /// strategy.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.get_current_time().await
    ///     })
    /// }).await;
    /// ```
    async fn with_ctx_read_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let guard = self.context().read().await;
        f(&*guard).await
    }

    /// Access context with write lock (async closure)
    ///
    /// # Example
    /// ```rust
    /// strategy.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.handle_strategy_command(cmd).await;
    ///     })
    /// }).await;
    /// ```
    async fn with_ctx_write_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a mut Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let mut guard = self.context().write().await;
        f(&mut *guard).await
    }
}
