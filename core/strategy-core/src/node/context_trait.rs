// std
use std::{collections::HashMap, fmt::Debug, sync::Arc};

// third-party
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use event_center_core::event::EventTrait;
use snafu::{IntoError, OptionExt};
use star_river_core::{
    custom_type::{CycleId, NodeId, NodeName, StrategyId},
    error::StarRiverErrorTrait,
};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

// current crate
use super::{metadata::NodeMetadata, utils::generate_default_output_handle_id};
use crate::{
    benchmark::node_benchmark::CompletedCycle,
    communication::{NodeCommandTrait, StrategyCommandTrait},
    error::{
        NodeError, NodeStateMachineError,
        node_error::{NodeEventSendFailedSnafu, OutputHandleNotFoundSnafu, StrategyCommandSendFailedSnafu},
    },
    event::{
        node::NodeEventTrait,
        node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
    },
    node::{
        NodeType,
        node_handles::{HandleId, NodeInputHandle, NodeOutputHandle},
        node_state_machine::{StateChangeActions, StateMachine},
    },
    strategy::cycle::Cycle,
};

// ============================================================================
// Metadata Trait: NodeMetadata
// ============================================================================

/// Node context core trait
///
/// All node contexts must implement this trait to provide access to base context
pub trait NodeMetaDataExt: Debug + Send + Sync + 'static {
    type StateMachine: StateMachine;
    type NodeEvent: NodeEventTrait + From<CommonEvent>;
    type NodeCommand: NodeCommandTrait;
    type StrategyCommand: StrategyCommandTrait;
    type Error: StarRiverErrorTrait;

    /// Get immutable reference to base context
    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand>;

    /// Get mutable reference to base context
    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand>;
}

// ============================================================================
// Extension Trait 1: NodeIdentity - Node identity information (read-only)
// ============================================================================

/// Node identity information extension
///
/// Provides access to read-only information such as node ID, name, type
pub trait NodeInfoExt: NodeMetaDataExt {
    /// Get cycle ID
    #[inline]
    fn cycle_id(&self) -> CycleId {
        self.metadata().cycle_id()
    }

    #[inline]
    fn cycle_watch_rx(&self) -> watch::Receiver<Cycle> {
        self.metadata().cycle_watch_rx()
    }

    #[inline]
    fn strategy_time(&self) -> DateTime<Utc> {
        self.metadata().strategy_time()
    }

    #[inline]
    fn strategy_time_watch_rx(&self) -> watch::Receiver<DateTime<Utc>> {
        self.metadata().strategy_time_watch_rx()
    }

    /// Get node ID
    #[inline]
    fn node_id(&self) -> &NodeId {
        self.metadata().node_id()
    }

    /// Get node name
    #[inline]
    fn node_name(&self) -> &NodeName {
        self.metadata().node_name()
    }

    /// Get node type
    #[inline]
    fn node_type(&self) -> &NodeType {
        self.metadata().node_type()
    }

    /// Get strategy ID
    #[inline]
    fn strategy_id(&self) -> StrategyId {
        self.metadata().strategy_id()
    }
}

// Automatically implement NodeIdentity for all types that implement NodeMetaDataTrait
impl<Ctx> NodeInfoExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// Extension Trait 2: NodeRelation - Node relationship management
// ============================================================================

/// Node relationship management extension
///
/// Manages topological relationships between nodes (upstream nodes, leaf nodes, etc.)
pub trait NodeRelationExt: NodeMetaDataExt {
    /// Add source node (upstream node) ID
    #[inline]
    fn add_source_node(&mut self, source_node_id: NodeId) {
        self.metadata_mut().source_nodes_mut().push(source_node_id);
    }

    /// Check if specified source node exists
    #[inline]
    fn has_source_node(&self, source_node_id: &str) -> bool {
        self.metadata().source_nodes().iter().any(|id| id == source_node_id)
    }

    /// Get number of source nodes
    #[inline]
    fn source_node_count(&self) -> usize {
        self.metadata().source_nodes().len()
    }

    /// Get all source node IDs
    #[inline]
    fn source_nodes(&self) -> &[NodeId] {
        self.metadata().source_nodes()
    }

    /// Set whether node is a leaf node
    #[inline]
    fn set_leaf_node(&mut self, is_leaf: bool) {
        self.metadata_mut().set_is_leaf_node(is_leaf);
    }

    /// Check if node is a leaf node
    #[inline]
    fn is_leaf_node(&self) -> bool {
        self.metadata().is_leaf_node()
    }
}

// Automatically implement NodeRelation for all types that implement NodeMetaDataTrait
impl<Ctx> NodeRelationExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// Extension Trait 3: NodeHandle - Handle management
// ============================================================================

/// Node handle management extension
///
/// Manages node input/output handles and strategy output handles
pub trait NodeHandleExt: NodeMetaDataExt + NodeInfoExt {
    fn set_output_handles(&mut self) -> Result<(), Self::Error>;

    /// Add input handle
    #[inline]
    fn add_input_handle(&mut self, input_handle: NodeInputHandle<Self::NodeEvent>) {
        self.metadata_mut().add_input_handle(input_handle);
    }

    /// Find input handle
    #[inline]
    fn find_input_handle(&self, input_handle_id: &str) -> Option<&NodeInputHandle<Self::NodeEvent>> {
        self.metadata()
            .input_handles()
            .iter()
            .find(|h| h.input_handle_id == input_handle_id)
    }

    /// Get all input handles
    #[inline]
    fn input_handles(&self) -> &[NodeInputHandle<Self::NodeEvent>] {
        self.metadata().input_handles()
    }

    // ------------------------------------------------------------------------
    // Output handle management
    // ------------------------------------------------------------------------

    #[inline]
    fn add_default_output_handle(&mut self, output_handle: NodeOutputHandle<Self::NodeEvent>) {
        self.metadata_mut().add_output_handle(output_handle);
    }

    /// Get default output handle
    #[inline]
    fn default_output_handle(&self) -> Result<&NodeOutputHandle<Self::NodeEvent>, NodeError> {
        let default_handle_id = generate_default_output_handle_id(self.node_id());
        self.metadata()
            .output_handles()
            .get(&default_handle_id)
            .context(OutputHandleNotFoundSnafu {
                node_name: self.node_name().clone(),
                handle_id: Some(default_handle_id),
                config_id: None,
            })
    }

    /// Check if default output handle exists
    #[inline]
    fn has_default_output_handle(&self) -> bool {
        let default_handle_id = format!("{}:default", self.node_id());
        self.metadata().output_handles().contains_key(&default_handle_id)
    }

    /// Add output handle
    #[inline]
    fn add_output_handle(&mut self, is_default: bool, config_id: i32, handle_id: HandleId, sender: broadcast::Sender<Self::NodeEvent>) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let handle = NodeOutputHandle::new(node_id, node_name, is_default, config_id, handle_id, sender);
        self.metadata_mut().add_output_handle(handle);
    }

    #[inline]
    fn output_handles(&self) -> &HashMap<HandleId, NodeOutputHandle<Self::NodeEvent>> {
        self.metadata().output_handles()
    }

    /// Get output handle
    #[inline]
    fn output_handle(&self, handle_id: &str) -> Result<&NodeOutputHandle<Self::NodeEvent>, NodeError> {
        self.metadata().output_handles().get(handle_id).context(OutputHandleNotFoundSnafu {
            node_name: self.node_name().clone(),
            handle_id: Some(handle_id.to_string()),
            config_id: None,
        })
    }

    /// Check if output handle exists
    #[inline]
    fn has_output_handle(&self, handle_id: &str) -> bool {
        self.metadata().output_handles().contains_key(handle_id)
    }

    /// Get strategy bound handle
    #[inline]
    fn strategy_bound_handle(&self) -> &NodeOutputHandle<Self::NodeEvent> {
        self.metadata().strategy_bound_handle()
    }

    #[inline]
    fn subscribe_strategy_bound_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<Self::NodeEvent> {
        self.metadata_mut().subscribe_strategy_bound_handle(subscriber_id)
    }

    fn subscribe_output_handle(
        &mut self,
        handle_id: String,
        subscriber_id: String,
    ) -> Result<(i32, broadcast::Receiver<Self::NodeEvent>), NodeError> {
        self.metadata_mut().subscribe_output_handle(handle_id, subscriber_id)
    }
}

// ============================================================================
// Extension Trait 4: NodeStateMachineOps - State machine operations
// ============================================================================

/// Node state machine operations extension
///
/// Manages node runtime state and state transitions
#[async_trait]
pub trait NodeStateMachineExt: NodeMetaDataExt {
    /// Get state machine reference
    fn state_machine(&self) -> Arc<RwLock<Self::StateMachine>> {
        self.metadata().state_machine()
    }

    /// Get current runtime state
    #[inline]
    async fn run_state(&self) -> <Self::StateMachine as StateMachine>::State {
        self.state_machine().read().await.current_state().clone()
    }

    /// Check if in specified state
    #[inline]
    async fn is_in_state(&self, state: &<Self::StateMachine as StateMachine>::State) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// State transition
    #[inline]
    async fn transition_state(
        &self,
        trigger: <Self::StateMachine as StateMachine>::Trigger,
    ) -> Result<
        StateChangeActions<<Self::StateMachine as StateMachine>::State, <Self::StateMachine as StateMachine>::Action>,
        NodeStateMachineError,
    > {
        self.state_machine().write().await.transition(trigger)
    }
}

// Automatically implement NodeStateMachineOps for all types that implement NodeMetaDataTrait
#[async_trait]
impl<Ctx> NodeStateMachineExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// Extension Trait 5: NodeCommunication - Communication management
// ============================================================================

/// Node communication management extension
///
/// Manages communication with strategy and other nodes, including command sending/receiving and event sending
#[async_trait]
pub trait NodeCommunicationExt: NodeMetaDataExt + NodeInfoExt + NodeRelationExt + NodeHandleExt {
    /// Get strategy command sender
    #[inline]
    fn strategy_command_sender(&self) -> &mpsc::Sender<Self::StrategyCommand> {
        self.metadata().strategy_command_sender()
    }

    async fn send_strategy_command(&self, command: Self::StrategyCommand) -> Result<(), NodeError> {
        self.strategy_command_sender().send(command).await.map_err(|e| {
            StrategyCommandSendFailedSnafu {
                node_name: self.node_name().clone(),
            }
            .into_error(Arc::new(e))
        })?;
        Ok(())
    }

    /// Get node command receiver
    #[inline]
    fn node_command_receiver(&self) -> Arc<Mutex<mpsc::Receiver<Self::NodeCommand>>> {
        self.metadata().node_command_receiver()
    }

    /// Send event to specified output handle
    ///
    /// # Arguments
    /// - `handle_id` - Output handle ID
    /// - `event` - Event to send
    fn output_handle_send(&self, event: Self::NodeEvent) -> Result<(), crate::error::NodeError> {
        let handle_id = event.output_handle_id().to_string();
        let output_handle = self.output_handle(&handle_id)?;

        if output_handle.is_connected() {
            output_handle.send(event).map_err(|e| {
                NodeEventSendFailedSnafu {
                    node_name: self.node_name().clone(),
                    handle_id: handle_id,
                }
                .into_error(Arc::new(e))
            })?;
        } else {
            // tracing::warn!(
            //     "@[{}] output handle {} is not connected, skip sending event",
            //     self.node_name(),
            //     handle_id
            // );
        }

        Ok(())
    }

    /// Send event to strategy bound handle
    ///
    /// # Arguments
    /// - `event` - Event to send
    fn strategy_bound_handle_send(&self, event: Self::NodeEvent) -> Result<(), NodeError> {
        let strategy_handle = self.strategy_bound_handle();

        strategy_handle.send(event)
    }

    /// Send event to default output handle
    ///
    /// # Arguments
    /// - `event` - Event to send
    fn default_output_handle_send(&self, event: Self::NodeEvent) -> Result<(), crate::error::NodeError> {
        let default_handle = self.default_output_handle()?;
        default_handle.send(event)
    }

    fn send_execute_over_event(
        &self,
        config_id: Option<i32>,
        context: Option<String>,
        datetime: Option<DateTime<Utc>>,
    ) -> Result<(), NodeError> {
        if !self.is_leaf_node() {
            return Ok(());
        }

        let payload = ExecuteOverPayload::new(config_id, context);
        let execute_over_event: CommonEvent = if let Some(datetime) = datetime {
            ExecuteOverEvent::new_with_time(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                self.strategy_bound_handle().output_handle_id().clone(),
                datetime,
                payload,
            )
            .into()
        } else {
            ExecuteOverEvent::new(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                self.strategy_bound_handle().output_handle_id().clone(),
                payload,
            )
            .into()
        };

        self.strategy_bound_handle_send(execute_over_event.into())?;

        Ok(())
    }

    // send trigger event to downstream node. if current node is leaf node, send execute over event instead.
    async fn send_trigger_event(
        &self,
        handle_id: &str,
        config_id: i32,
        context: Option<String>,
        datetime: Option<DateTime<Utc>>,
    ) -> Result<(), NodeError> {
        // Leaf nodes do not send trigger events
        if self.is_leaf_node() {
            // self.send_execute_over_event()?;
            return Ok(());
        }

        let payload = TriggerPayload::new(config_id, context);

        let trigger_event: CommonEvent = if let Some(datetime) = datetime {
            TriggerEvent::new_with_time(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                handle_id.to_string(),
                datetime,
                payload,
            )
            .into()
        } else {
            TriggerEvent::new(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                handle_id.to_string(),
                payload,
            )
            .into()
        };

        let output_handle = self.output_handle(handle_id)?;
        output_handle.send(trigger_event.into())
    }

    async fn default_output_handle_send_trigger_event(
        &self,
        config_id: i32,
        context: Option<String>,
        datetime: Option<DateTime<Utc>>,
    ) -> Result<(), NodeError> {
        let default_handle = self.default_output_handle()?;

        let payload = TriggerPayload::new(config_id, context);
        let trigger_event: CommonEvent = if let Some(datetime) = datetime {
            TriggerEvent::new_with_time(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                default_handle.output_handle_id().clone(),
                datetime,
                payload,
            )
            .into()
        } else {
            TriggerEvent::new(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().to_string(),
                default_handle.output_handle_id().clone(),
                payload,
            )
            .into()
        };

        self.default_output_handle_send(trigger_event.into())
    }
}

// Automatically implement NodeControl for all types that implement NodeMetaDataTrait
impl<Ctx> NodeCommunicationExt for Ctx where Ctx: NodeMetaDataExt + NodeInfoExt + NodeRelationExt + NodeHandleExt {}

// ============================================================================
// Extension Trait 6: NodeControl - Node runtime control
// ============================================================================

/// Node runtime control extension
///
/// Provides node runtime control functionality (cancellation, pause, etc.)
pub trait NodeTaskControlExt: NodeMetaDataExt {
    /// Get cancellation token
    #[inline]
    fn cancel_token(&self) -> &CancellationToken {
        self.metadata().cancel_token()
    }

    /// Check if cancelled
    #[inline]
    fn is_cancelled(&self) -> bool {
        self.cancel_token().is_cancelled()
    }

    /// Request cancellation
    #[inline]
    fn request_cancel(&self) {
        self.cancel_token().cancel();
    }
}

// Automatically implement NodeControl for all types that implement NodeMetaDataTrait
impl<Ctx> NodeTaskControlExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// Extension Trait 7: NodeEventHandler - Event handling (requires concrete implementation)
// ============================================================================

/// Node event handling extension
///
/// Defines how node handles various events, requires concrete node types to implement
#[async_trait]
pub trait NodeEventHandlerExt: NodeMetaDataExt {
    type EngineEvent: EventTrait;

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) -> Result<(), Self::Error>;
    /// Handle node events
    ///
    /// Default implementation only logs, concrete nodes should override this method
    async fn handle_source_node_event(&mut self, node_event: Self::NodeEvent) -> Result<(), Self::Error>;

    /// Handle node commands
    ///
    /// Default implementation only logs, concrete nodes should override this method
    async fn handle_command(&mut self, node_command: Self::NodeCommand);
}

// Note: NodeEventHandler does not provide automatic implementation, as it requires concrete node types to implement based on business logic

// ============================================================================
// Extension Trait 8: NodeBenchmark - Performance statistics (provides default implementation)
// ============================================================================

/// Node performance statistics extension
///
/// Provides functionality to send performance statistics data to strategy layer
#[async_trait]
pub trait NodeBenchmarkExt: NodeMetaDataExt + NodeInfoExt + NodeCommunicationExt {
    /// Mount node cycle tracker data
    ///
    /// Sends node performance statistics data to strategy layer
    ///
    /// # Arguments
    /// - `node_id` - Node ID
    /// - `node_name` - Node name
    /// - `cycle_tracker` - Cycle tracker data
    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error>;
}

// ============================================================================
// Composite Trait: StrategyNodeContext (collection of all functionality)
// ============================================================================

/// Complete strategy node context trait
///
/// Combines all node context required functionality, providing complete capability set for nodes
pub trait NodeContextExt:
    NodeMetaDataExt
    + NodeInfoExt
    + NodeRelationExt
    + NodeHandleExt
    + NodeStateMachineExt
    + NodeCommunicationExt
    + NodeTaskControlExt
    + NodeEventHandlerExt
    + NodeBenchmarkExt
{
}

// Automatically implement StrategyNodeContext for all types that satisfy all constraints
impl<Ctx> NodeContextExt for Ctx where
    Ctx: NodeMetaDataExt
        + NodeInfoExt
        + NodeRelationExt
        + NodeHandleExt
        + NodeStateMachineExt
        + NodeCommunicationExt
        + NodeTaskControlExt
        + NodeEventHandlerExt
        + NodeBenchmarkExt
{
}
