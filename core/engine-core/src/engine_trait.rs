use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
// use event_center::EventCenterSingleton;
use event_center::EventCenterSingleton;
use futures::{StreamExt, stream::select_all};
use star_river_core::error::StarRiverErrorTrait;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;

use super::EngineEventReceiver;
use crate::{
    context_trait::{EngineContextTrait, EngineEventHandler},
    state_machine::{EngineAction, EngineStateTransTrigger},
};

// Empty trait, only used for marker
pub trait Engine: Send + Sync + 'static {}

// ============================================================================
// Engine Context Accessor Trait
// ============================================================================

/// Engine context accessor trait
///
/// # Associated Types
/// - `Context`: Engine context type, must implement `EngineContextTrait` with its `Action` associated type matching `Self::Action`
/// - `Action`: Engine action type, must implement `EngineAction`
#[async_trait]
pub trait EngineContextAccessor: Send + Sync {
    /// Engine context type
    type Context: EngineContextTrait<Action = Self::Action>;

    /// Engine action type
    type Action: EngineAction;

    type Error: StarRiverErrorTrait;

    /// Get reference to context
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// Access context with read lock (synchronous closure)
    ///
    /// # Example
    /// ```rust,ignore
    /// let engine_name = engine.with_ctx_read(|ctx| {
    ///     ctx.engine_name().clone()
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
    /// ```rust,ignore
    /// engine.with_ctx_write(|ctx| {
    ///     // Synchronous operations
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
    /// ```rust,ignore
    /// engine.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.some_async_operation().await
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
    /// ```rust,ignore
    /// engine.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.handle_command(cmd).await;
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
// Engine Lifecycle Trait
// ============================================================================

/// Engine lifecycle management trait
///
/// Defines engine lifecycle operations (start, stop, state update)
/// Depends on `EngineContextAccessor` to access context
///
/// # Associated Types
/// - `Error`: Engine-specific error type, must implement `StarRiverErrorTrait`
#[async_trait]
pub trait EngineLifecycle: EngineContextAccessor {
    /// Start the engine
    ///
    /// Called before the engine begins running, used to perform necessary initialization
    /// Specific engines can override this method to implement custom startup logic
    async fn start(&self) -> Result<(), Self::Error>;

    /// Stop the engine
    ///
    /// Gracefully stop the engine and clean up resources
    /// Specific engines can override this method to implement custom cleanup logic
    async fn stop(&self) -> Result<(), Self::Error>;

    /// Update engine state
    ///
    /// Handle engine state transition events
    async fn update_engine_state(&self, trans_trigger: EngineStateTransTrigger) -> Result<(), Self::Error>;
}

// ============================================================================
// Engine Event Listener Trait
// ============================================================================

/// Engine event listener trait
///
/// Requires context type to implement both `EngineContextTrait` and `EngineEventHandler`
#[async_trait]
pub trait EngineEventListener: EngineContextAccessor
where
    Self::Context: EngineEventHandler,
{
    /// Listen to external events
    async fn listen_events(&self) {
        // Use with_ctx_read_async to handle lifetimes correctly
        let (engine_name, event_receivers) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let engine_name = ctx.engine_name().clone(); // Clone to avoid lifetime issues
                    let should_receive_channels = EngineEventReceiver::get_event_receivers(&engine_name);
                    let mut event_receivers = Vec::new();
                    for channel in should_receive_channels.iter() {
                        let event_receiver = EventCenterSingleton::subscribe(channel).await.unwrap();
                        event_receivers.push(event_receiver);
                    }
                    (engine_name, event_receivers)
                })
            })
            .await;

        if event_receivers.is_empty() {
            tracing::warn!("{}: no event receivers", engine_name);
            return;
        }

        let streams: Vec<_> = event_receivers.into_iter().map(|receiver| BroadcastStream::new(receiver)).collect();

        let mut combined_stream = select_all(streams);
        let context = self.context().clone();

        tracing::debug!("#[{}]: start listening events", engine_name);

        tokio::spawn(async move {
            loop {
                if let Some(received_event) = combined_stream.next().await {
                    match received_event {
                        Ok(event) => {
                            let mut context_guard = context.write().await;
                            // tracing::debug!("{}: received event: {:?}", engine_name, event);
                            context_guard.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::error!("#[{}]: receive event error: {}", engine_name, e);
                        }
                    }
                }
            }
        });
    }

    /// Listen to engine commands
    async fn listen_commands(&self) {
        let (engine_name, command_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let engine_name = ctx.engine_name().clone(); // Clone to avoid lifetime issues
                    let command_receiver = EventCenterSingleton::command_receiver(&engine_name.clone().into()).await.unwrap();

                    (engine_name, command_receiver)
                })
            })
            .await;
        tracing::debug!("#[{}]: start listening commands", engine_name);

        let context = self.context().clone();
        tokio::spawn(async move {
            loop {
                if let Some(received_command) = command_receiver.lock().await.recv().await {
                    let mut context_guard = context.write().await;
                    context_guard.handle_command(received_command).await;
                }
            }
        });
    }
}

impl<T> EngineEventListener for T
where
    T: EngineContextAccessor,
    T::Context: EngineEventHandler,
{
}
