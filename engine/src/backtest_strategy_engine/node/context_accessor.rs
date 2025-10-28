use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::node_context::BacktestNodeContextTrait;
use super::BacktestNodeTrait;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::*;
use snafu::OptionExt;

/// Trait that exposes helper methods for safely accessing the shared node context.
#[async_trait]
pub trait BacktestNodeContextAccessor: Send + Sync {
    /// Returns the shared context used by a node.
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>;

    /// Executes the provided closure with read access to the typed context.
    async fn with_ctx_read<T, R>(&self, f: impl for<'ctx> FnOnce(&'ctx T) -> R + Send) -> Result<R, BacktestStrategyNodeError>
    where
        T: BacktestNodeContextTrait + 'static,
        R: Send + 'async_trait,
    {
        let guard = self.get_context().read_owned().await;
        let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
        let ctx = ctx_ref
            .as_any()
            .downcast_ref::<T>()
            .context(ContextDowncastFailedSnafu{
                expect_type: std::any::type_name::<T>(),
                actual_type: std::any::type_name_of_val(ctx_ref),
            })?;
        Ok(f(ctx))
    }

    /// Executes the provided closure with write access to the typed context.
    async fn with_ctx_write<T, R>(&self, f: impl for<'ctx> FnOnce(&'ctx mut T) -> R + Send) -> Result<R, BacktestStrategyNodeError>
    where
        T: BacktestNodeContextTrait + 'static,
        R: Send + 'async_trait,
    {
        let mut guard = self.get_context().write_owned().await;
        let result = {
            let actual_type = {
                let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
                std::any::type_name_of_val(ctx_ref)
            };
            let ctx_mut: &mut dyn BacktestNodeContextTrait = &mut **guard;
            let ctx = ctx_mut
                .as_any_mut()
                .downcast_mut::<T>()
                .context(ContextDowncastFailedSnafu{
                    expect_type: std::any::type_name::<T>(),
                    actual_type,
                })?;
            Ok(f(ctx))
        };
        result
    }

    /// Executes the provided closure with read access to the untyped context.
    async fn with_ctx_read_dyn<R>(
        &self,
        f: impl for<'ctx> FnOnce(&'ctx dyn BacktestNodeContextTrait) -> R + Send,
    ) -> R
    where
        R: Send + 'async_trait,
    {
        let guard = self.get_context().read_owned().await;
        let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
        f(ctx_ref)
    }

    /// Executes the provided closure with write access to the untyped context.
    async fn with_ctx_write_dyn<R>(
        &self,
        f: impl for<'ctx> FnOnce(&'ctx mut dyn BacktestNodeContextTrait) -> R + Send,
    ) -> R
    where
        R: Send + 'async_trait,
    {
        let mut guard = self.get_context().write_owned().await;
        let ctx_mut: &mut dyn BacktestNodeContextTrait = &mut **guard;
        f(ctx_mut)
    }

    /// Executes an asynchronous closure while holding the read lock on the untyped context.
    async fn with_ctx_read_async_dyn<R>(
        &self,
        f: impl for<'ctx> FnOnce(
                &'ctx dyn BacktestNodeContextTrait,
            ) -> Pin<Box<dyn Future<Output = R> + Send + 'ctx>>
            + Send
            + 'async_trait,
    ) -> R
    where
        R: Send + 'async_trait,
    {
        let guard = self.get_context().read_owned().await;
        let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
        f(ctx_ref).await
    }

    /// Executes an asynchronous closure while holding the write lock on the untyped context.
    async fn with_ctx_write_async_dyn<R>(
        &self,
        f: impl for<'ctx> FnOnce(
                &'ctx mut dyn BacktestNodeContextTrait,
            ) -> Pin<Box<dyn Future<Output = R> + Send + 'ctx>>
            + Send
            + 'async_trait,
    ) -> R
    where
        R: Send + 'async_trait,
    {
        let mut guard = self.get_context().write_owned().await;
        let ctx_mut: &mut dyn BacktestNodeContextTrait = &mut **guard;
        f(ctx_mut).await
    }

    /// Executes an asynchronous closure while holding the read lock on the typed context.
    async fn with_ctx_read_async<T, R>(
        &self,
        f: impl for<'ctx> FnOnce(&'ctx T) -> Pin<Box<dyn Future<Output = R> + Send + 'ctx>> + Send + 'async_trait,
    ) -> Result<R, BacktestStrategyNodeError>
    where
        T: BacktestNodeContextTrait + 'static,
        R: Send + 'async_trait,
    {
        let guard = self.get_context().read_owned().await;
        let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
        let ctx = ctx_ref
            .as_any()
            .downcast_ref::<T>()
            .context(ContextDowncastFailedSnafu{
                expect_type: std::any::type_name::<T>(),
                actual_type: std::any::type_name_of_val(ctx_ref),
            })?;
        Ok(f(ctx).await)
    }

    /// Executes an asynchronous closure while holding the write lock on the typed context.
    async fn with_ctx_write_async<T, R>(
        &self,
        f: impl for<'ctx> FnOnce(&'ctx mut T) -> Pin<Box<dyn Future<Output = R> + Send + 'ctx>> + Send + 'async_trait,
    ) -> Result<R, BacktestStrategyNodeError>
    where
        T: BacktestNodeContextTrait + 'static,
        R: Send + 'async_trait,
    {
        let mut guard = self.get_context().write_owned().await;
        let result = {
            let actual_type = {
                let ctx_ref: &dyn BacktestNodeContextTrait = &**guard;
                std::any::type_name_of_val(ctx_ref)
            };
            let ctx_mut: &mut dyn BacktestNodeContextTrait = &mut **guard;
            let ctx = ctx_mut
                .as_any_mut()
                .downcast_mut::<T>()
                .context(ContextDowncastFailedSnafu{
                    expect_type: std::any::type_name::<T>(),
                    actual_type,
                })?;
            Ok(f(ctx).await)
        };
        result
    }
}

impl<T> BacktestNodeContextAccessor for T
where
    T: BacktestNodeTrait + ?Sized,
{
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        BacktestNodeTrait::get_context(self)
    }
}
