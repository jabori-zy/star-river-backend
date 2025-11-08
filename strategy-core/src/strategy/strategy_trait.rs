// std
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
};

// third-party
use async_trait::async_trait;
use tokio::sync::RwLock;
use star_river_core::error::StarRiverErrorTrait;

// current crate
use super::context_trait::StrategyMetaDataExt;
use super::state_machine::StrategyStateTransTrigger;


// ============================================================================
// Strategy Context Accessor (provides convenient read/write lock access methods)
// ============================================================================

/// Strategy context accessor
///
/// Provides convenient context access methods for generic strategies,
/// encapsulates read/write lock acquisition and release logic, avoiding boilerplate code
#[async_trait]
pub trait StrategyContextAccessor: Send + Sync {
    /// Context type, must implement StrategyMetaDataExt
    type Context: StrategyMetaDataExt;

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
    async fn with_ctx_read_async<R>(&self, f: impl for<'a> FnOnce(&'a Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send) -> R
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
    async fn with_ctx_write_async<R>(&self, f: impl for<'a> FnOnce(&'a mut Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send) -> R
    where
        R: Send,
    {
        let mut guard = self.context().write().await;
        f(&mut *guard).await
    }
}

// ============================================================================
// Strategy Lifecycle Management
// ============================================================================

/// Strategy lifecycle management trait
///
/// Defines lifecycle-related operations for strategies (check, initialization, stop)
/// Depends on `StrategyContextAccessor` to access context
#[async_trait]
pub trait StrategyLifecycle: StrategyContextAccessor {

    type Trigger: StrategyStateTransTrigger;

    /// Error type
    type Error: StarRiverErrorTrait;

    /// Initialize strategy
    ///
    /// Initializes the strategy and all its nodes in preparation for execution.
    /// This should be called after the strategy check passes.
    ///
    /// # State transitions
    /// - CheckPassed -> Initializing -> Ready
    /// - On error: any state -> Failed
    ///
    /// # Returns
    /// - `Ok(())`: Strategy initialized successfully
    /// - `Err(Self::Error)`: Initialization failed
    async fn init_strategy(&mut self) -> Result<(), Self::Error>;

    /// Stop strategy
    ///
    /// Gracefully stops the strategy and all its nodes.
    /// Waits for all nodes to reach stopped state before completing.
    ///
    /// # State transitions
    /// - Ready/Running -> Stopping -> Stopped
    /// - On error: any state -> Failed
    ///
    /// # Returns
    /// - `Ok(())`: Strategy stopped successfully
    /// - `Err(Self::Error)`: Stop failed or timeout
    async fn stop_strategy(&mut self) -> Result<(), Self::Error>;

    /// Update strategy state
    ///
    /// Handles strategy state transitions and executes associated actions.
    /// This is the core state machine orchestration method that:
    /// 1. Triggers state transitions based on the provided trigger event
    /// 2. Executes all actions associated with the state transition
    /// 3. Updates the state machine in the context
    ///
    /// # Arguments
    /// - `trigger`: The event that triggers the state transition
    ///
    /// # Returns
    /// - `Ok(())`: State transition completed successfully
    /// - `Err(Self::Error)`: State transition or action execution failed
    ///
    /// # Example
    /// ```ignore
    /// // Trigger initialization
    /// strategy.update_strategy_state(StrategyStateTransTrigger::Initialize).await?;
    /// ```
    async fn update_strategy_state(&mut self, trigger: Self::Trigger) -> Result<(), Self::Error>;
}

// ============================================================================
// Strategy Event Listener
// ============================================================================

/// Strategy event listener trait
///
/// Defines methods for listening to various events (node events, strategy commands, stats events)
/// Depends on `StrategyContextAccessor` to access context
#[async_trait]
pub trait StrategyEventListener: StrategyContextAccessor {
    /// Listen to node events
    ///
    /// Subscribes to events from all nodes in the workflow and handles them in a background task.
    /// All node events are aggregated and processed by the strategy.
    async fn listen_node_events(&self);

    /// Listen to strategy commands
    ///
    /// Listens for commands sent to the strategy and processes them in a background task.
    async fn listen_strategy_command(&self);

    // / Listen to strategy statistics events
    // /
    // / Listens for performance statistics and metrics updates in a background task.
    // async fn listen_strategy_stats_event(&self);
}
