// std
use std::{future::Future, pin::Pin, sync::Arc};

// third-party
use async_trait::async_trait;
use star_river_core::{custom_type::NodeId, error::StarRiverErrorTrait};
use tokio::sync::RwLock;

// current crate
use super::{
    context_trait::{
        NodeCommunicationExt, NodeContextExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeMetaDataExt, NodeTaskControlExt,
    },
    node_state_machine::StateTransTrigger,
};
// workspace crate
use crate::event::node_common_event::{CommonEvent, NodeRunningLogEvent};

#[async_trait]
pub trait NodeTrait: Clone + Send + Sync + 'static {
    async fn node_id(&self) -> NodeId;
}

// ============================================================================
// Node context accessor (provides convenient read/write lock access methods)
// ============================================================================

/// Node context accessor
///
/// Provides convenient context access methods for generic nodes,
/// encapsulates read/write lock acquisition and release logic, avoiding boilerplate code
#[async_trait]
pub trait NodeContextAccessor: Send + Sync {
    /// Context type, must implement NodeMetaDataTrait
    type Context: NodeMetaDataExt;

    /// Get shared reference to context
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// Access context with read lock (synchronous closure)
    ///
    /// # Example
    /// ```rust
    /// let node_name = node.with_ctx_read(|ctx| {
    ///     ctx.node_name().to_string()
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
    /// node.with_ctx_write(|ctx| {
    ///     ctx.set_leaf_node(true);
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
    /// node.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.send_execute_over_event().await
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
    /// node.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.handle_node_command(cmd).await;
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
// Node lifecycle management
// ============================================================================

/// Node lifecycle management trait
///
/// Defines node lifecycle operations (initialization, stop, state update)
/// Depends on `NodeContextAccessor` to access context
#[async_trait]
pub trait NodeLifecycle: NodeContextAccessor {
    /// Error type
    type Error: StarRiverErrorTrait;

    /// State transition trigger type
    type Trigger: StateTransTrigger;

    /// Initialize node
    ///
    /// Called before node starts running, used to perform necessary initialization
    /// Concrete nodes can override this method to implement custom initialization logic
    async fn init(&self) -> Result<(), Self::Error>;

    /// Stop node
    ///
    /// Gracefully stop node and cleanup resources
    /// Concrete nodes can override this method to implement custom cleanup logic
    async fn stop(&self) -> Result<(), Self::Error>;

    /// Update node state
    ///
    /// Handle node state transition events
    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error>;
}

// ============================================================================
// Node event listener
// ============================================================================

/// Node event listener trait
///
/// Defines methods for node to listen to various events (external events, upstream node events, strategy commands)
/// Depends on `NodeContextAccessor` to access context
#[async_trait]
pub trait NodeEventListener: NodeContextAccessor
where
    Self::Context: NodeContextExt,
{
    /// Listen to external events (engine events)
    ///
    /// Subscribe to corresponding event channels based on node type, and handle received events in background task
    async fn listen_engine_event(&self) {
        // TODO: Implement engine event listening logic
        // Need to implement subscription logic based on specific engine event system
        tracing::warn!("listen_engine_event");
    }

    /// Listen to node events (messages from upstream nodes)
    ///
    /// Listen to node messages received through input handles
    async fn listen_source_node_events(&self) {
        use futures::{StreamExt, stream::select_all};
        use tokio_stream::wrappers::BroadcastStream;

        let (input_handles, cancel_token, node_name) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let input_handles = ctx.input_handles().to_vec();
                    let cancel_token = ctx.cancel_token().clone();
                    let node_name = ctx.node_name().to_string();
                    (input_handles, cancel_token, node_name)
                })
            })
            .await;

        if input_handles.is_empty() {
            tracing::warn!("@[{}] have no input handles", node_name);
            return;
        }

        // Create a stream to receive messages passed from nodes
        let streams: Vec<_> = input_handles
            .iter()
            .map(|input_handle| BroadcastStream::new(input_handle.receiver()))
            .collect();

        let mut combined_stream = select_all(streams);
        let context = self.context().clone();

        tracing::debug!("@[{}] start to listen source node events", node_name);
        // Node receives data
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // If cancellation signal is triggered, abort task
                    _ = cancel_token.cancelled() => {
                        tracing::info!("@[{}] source node events listener task cancelled", node_name);
                        break;
                    }
                    // Receive message
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(message)) => {
                                // tracing::debug!("{} received message: {:?}", node_id, message);
                                let mut context_guard = context.write().await;
                                let handle_result = context_guard.handle_source_node_event(message).await;
                                if let Err(e) = handle_result {
                                    e.report_log();
                                    let current_time = context_guard.strategy_time();
                                    let running_error_log: CommonEvent = NodeRunningLogEvent::error_with_time(
                                        context_guard.cycle_id().clone(),
                                        context_guard.strategy_id().clone(),
                                        context_guard.node_id().clone(),
                                        context_guard.node_name().clone(),
                                        &e,
                                        current_time,
                                    ).into();
                                    if let Err(e) = context_guard.strategy_bound_handle_send(running_error_log.into()) {
                                        e.report_log();
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("@[{}] receive source node event error: {}", node_name, e);
                            }
                            None => {
                                tracing::warn!("@[{}] all source node event streams are closed", node_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    /// Listen to strategy commands
    ///
    /// Listen to control commands from strategy layer
    async fn listen_command(&self) {
        let (node_command_receiver, cancel_token, node_name) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let receiver = ctx.node_command_receiver();
                    let cancel_token = ctx.cancel_token().clone();
                    let node_name = ctx.node_name().to_string();
                    (receiver, cancel_token, node_name)
                })
            })
            .await;

        let context = self.context().clone();
        tracing::debug!("@[{}] start to listen command", node_name);
        // Node receives data
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // If cancellation signal is triggered, abort task
                    _ = cancel_token.cancelled() => {
                        tracing::info!("@[{}] command listener task cancelled", node_name);
                        break;
                    }

                    _ = async {
                        let mut command_receiver_guard = node_command_receiver.lock().await;

                        if let Some(received_command) = command_receiver_guard.recv().await {
                            let mut context_guard = context.write().await;
                            context_guard.handle_command(received_command).await;
                        }
                    } => {}
                }
            }
        });
    }
}

// Automatically implement NodeEventListener for all types that implement NodeContextAccessor and satisfy constraints
impl<T> NodeEventListener for T
where
    T: NodeContextAccessor,
    T::Context: super::context_trait::NodeContextExt,
{
}
