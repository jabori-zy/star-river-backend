use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

/// Virtual Trading System Context Accessor
///
/// Provides convenient methods to access context with read/write locks,
/// encapsulating lock acquisition and release logic to avoid boilerplate code
#[async_trait]
pub trait VtsCtxAccessor: Send + Sync {
    /// Context type
    type Context: Debug + Send + Sync + 'static;

    /// Get shared reference to context
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// Access context with read lock (synchronous closure)
    ///
    /// # Example
    /// ```rust
    /// let value = vts.with_ctx_read(|ctx| {
    ///     // Access context fields
    ///     ctx.some_field.clone()
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

    /// Access context with write lock (synchronous closure)
    ///
    /// # Example
    /// ```rust
    /// vts.with_ctx_write(|ctx| {
    ///     // Modify context fields
    ///     ctx.some_field = new_value;
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

    /// Access context with read lock (asynchronous closure)
    ///
    /// # Example
    /// ```rust
    /// vts.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.some_async_method().await
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

    /// Access context with write lock (asynchronous closure)
    ///
    /// # Example
    /// ```rust
    /// vts.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.some_async_mutation().await;
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

// ============================================================================
// VTS Event Handler - Event handling (requires concrete implementation)
// ============================================================================

/// Virtual Trading System Event Handler trait
///
/// Defines how VTS handles various events, requires concrete VTS type implementation
#[async_trait]
pub trait VTSEventHandler: Send + Sync {
    /// K-line event type
    type KlineEvent: Clone + Send + Sync + 'static + Debug;

    /// Handle K-line node event
    ///
    /// Custom implementation for each type
    async fn handle_kline_event(&mut self, event: Self::KlineEvent);
}

// ============================================================================
// VTS Event Listener - Event listening
// ============================================================================

/// Virtual Trading System Event Listener trait
///
/// Defines methods for listening to external events in the virtual trading system
/// Depends on `VTSAccessor` to access context
#[async_trait]
pub trait VTSEventListener: VtsCtxAccessor {
    /// Listen to K-line node events
    ///
    /// Listens to events from all K-line node receivers and processes them
    /// Reference: strategy-core/src/node/node_trait.rs listen_source_node_events
    async fn listen_kline_node_events(&self);

    /// Listen to VTS command events
    ///
    /// Listens to events from all VTS command receivers and processes them
    async fn listen_vts_command(&self);
}
