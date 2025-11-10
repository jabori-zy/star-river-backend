use std::collections::HashMap;
// std
use std::{fmt::Debug, sync::Arc};

// third-party
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use event_center_core::event::EventTrait;
use heartbeat::Heartbeat;
use petgraph::{Directed, Direction, Graph, graph::NodeIndex};
use sea_orm::DatabaseConnection;
use snafu::OptionExt;
use star_river_core::{
    custom_type::{NodeId, NodeName, StrategyId, StrategyName},
    error::StarRiverErrorTrait,
};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_util::sync::CancellationToken;

// current crate
use super::metadata::StrategyMetadata;
use crate::{
    benchmark::{
        StrategyBenchmark,
        node_benchmark::CompletedCycle,
        strategy_benchmark::{StrategyCycleTracker, StrategyPerformanceReport},
    },
    communication::{NodeCommandTrait, StrategyCommandTrait},
    error::{
        StrategyError, StrategyStateMachineError,
        strategy_error::{CustomVariableNotExistSnafu, NodeCycleDetectedSnafu},
    },
    event::node::NodeEventTrait,
    node::NodeTrait,
    node_infra::variable_node::variable_operation::UpdateVarValueOperation,
    strategy::{
        StrategyConfig,
        state_machine::{StrategyStateChangeActions, StrategyStateMachine},
    },
    variable::{
        StrategyVariable,
        custom_variable::{CustomVariable, VariableValue},
        sys_varibale::SysVariable,
        variable_operation::apply_variable_operation,
    },
};

// ============================================================================
// Metadata Trait StrategyMetadata
// ============================================================================

/// Strategy context core trait
///
/// All strategy contexts must implement this trait to provide access to base context
pub trait StrategyMetaDataExt: Debug + Send + Sync + 'static {
    type Node: NodeTrait;
    type StateMachine: StrategyStateMachine;
    type StrategyCommand: StrategyCommandTrait;
    type NodeCommand: NodeCommandTrait;

    /// Get immutable reference to base context
    fn metadata(&self) -> &StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand>;

    /// Get mutable reference to base context
    fn metadata_mut(&mut self) -> &mut StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand>;
}

// ============================================================================
// Extension Trait 1: StrategyIdentity - Strategy identity information (read-only)
// ============================================================================

/// Strategy identity information extension
///
/// Provides access to read-only information such as strategy ID and name
pub trait StrategyIdentityExt: StrategyMetaDataExt {
    /// Get strategy config
    #[inline]
    fn strategy_config(&self) -> &StrategyConfig {
        self.metadata().strategy_config()
    }

    /// Get strategy ID
    #[inline]
    fn strategy_id(&self) -> StrategyId {
        self.metadata().strategy_id()
    }

    /// Get strategy name
    #[inline]
    fn strategy_name(&self) -> &StrategyName {
        self.metadata().strategy_name()
    }
}

// Automatically implement StrategyIdentity for all types that implement StrategyMetaDataExt
impl<Ctx> StrategyIdentityExt for Ctx where Ctx: StrategyMetaDataExt {}

#[async_trait]
pub trait StrategyInfoExt: StrategyMetaDataExt {
    async fn current_time(&self) -> DateTime<Utc> {
        self.metadata().current_time().await
    }

    async fn set_current_time(&mut self, current_time: DateTime<Utc>) {
        self.metadata_mut().set_current_time(current_time).await;
    }
}

impl<Ctx> StrategyInfoExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 2: StrategyWorkflow - Workflow management (topological sort)
// ============================================================================

/// Strategy workflow management extension
///
/// Provides workflow management capabilities such as topological sorting and node lifecycle management
#[async_trait]
pub trait StrategyWorkflowExt: StrategyMetaDataExt + StrategyIdentityExt {
    /// Error type
    type Error: StarRiverErrorTrait;

    /// Perform topological sort on the workflow graph
    ///
    /// Returns nodes in topological order, ensuring dependencies are processed before dependents
    ///
    /// # Returns
    /// - `Ok(Vec<Self::Node>)`: Sorted list of nodes in topological order
    /// - `Err(Self::Error)`: Error if cycle is detected or sort fails
    ///
    /// # Example
    /// ```ignore
    /// let sorted_nodes = strategy_context.topological_sort()?;
    /// for node in sorted_nodes {
    ///     // Process nodes in topological order
    /// }
    /// ```
    fn topological_sort(&self) -> Result<Vec<Self::Node>, StrategyError> {
        let result = petgraph::algo::toposort(&self.metadata().graph(), None);
        match result {
            Ok(nodes_index) => Ok(nodes_index
                .into_iter()
                .map(|index| self.metadata().graph()[index].clone())
                .collect()),
            Err(_) => {
                let error = NodeCycleDetectedSnafu {
                    strategy_name: self.strategy_name().clone(),
                }
                .build();
                return Err(error);
            }
        }
    }

    /// Initialize all nodes in the workflow
    ///
    /// Initializes nodes in topological order to ensure dependencies are initialized first.
    /// Receives Arc<RwLock<Self>> to control lock acquisition and release internally, avoiding deadlocks.
    ///
    /// # Arguments
    /// - `context`: Arc-wrapped RwLock of the strategy context
    ///
    /// # Returns
    /// - `Ok(())`: All nodes initialized successfully
    /// - `Err(Self::Error)`: Node initialization failed
    ///
    /// # Example
    /// ```ignore
    /// let context = Arc::new(RwLock::new(strategy_context));
    /// StrategyContext::init_node(context.clone()).await?;
    /// ```
    async fn init_node(context: Arc<RwLock<Self>>) -> Result<(), Self::Error>
    where
        Self: Sized;

    /// Stop all nodes in the workflow
    ///
    /// Stops nodes in topological order to ensure proper cleanup.
    /// Receives Arc<RwLock<Self>> to control lock acquisition and release internally, avoiding deadlocks.
    ///
    /// # Arguments
    /// - `context`: Arc-wrapped RwLock of the strategy context
    ///
    /// # Returns
    /// - `Ok(())`: All nodes stopped successfully
    /// - `Err(Self::Error)`: Node stop failed
    ///
    /// # Example
    /// ```ignore
    /// let context = Arc::new(RwLock::new(strategy_context));
    /// StrategyContext::stop_node(context.clone()).await?;
    /// ```
    async fn stop_node(context: Arc<RwLock<Self>>) -> Result<(), Self::Error>
    where
        Self: Sized;

    fn node(&self, node_index: NodeIndex) -> Option<&Self::Node> {
        self.metadata().graph().node_weight(node_index)
    }

    fn node_mut(&mut self, node_index: NodeIndex) -> Option<&mut Self::Node> {
        self.metadata_mut().graph_mut().node_weight_mut(node_index)
    }

    fn get_leaf_node_indexs(&self) -> Vec<NodeIndex> {
        self.metadata().graph().externals(Direction::Outgoing).collect()
    }

    fn graph(&self) -> &Graph<Self::Node, (), Directed> {
        self.metadata().graph()
    }

    fn graph_mut(&mut self) -> &mut Graph<Self::Node, (), Directed> {
        self.metadata_mut().graph_mut()
    }

    fn node_indices(&self) -> &HashMap<NodeId, NodeIndex> {
        self.metadata().node_indices()
    }

    fn get_node_index(&self, node_id: &str) -> Option<&NodeIndex> {
        self.metadata().get_node_index(node_id)
    }

    fn get_node(&self, node_id: &str) -> Option<&Self::Node> {
        let node_index = self.get_node_index(node_id)?;
        self.metadata().graph().node_weight(*node_index)
    }

    fn leaf_node_ids(&self) -> &Vec<NodeId> {
        self.metadata().leaf_node_ids()
    }

    fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.metadata_mut().set_leaf_node_ids(leaf_node_ids);
    }

    async fn add_node(&mut self, node: Self::Node) -> NodeIndex {
        let node_id = node.node_id().await;
        let node_index = self.metadata_mut().graph_mut().add_node(node);
        self.metadata_mut().node_indices_mut().insert(node_id.to_string(), node_index);
        node_index
    }
}

// ============================================================================
// Extension Trait 3: StrategyEventHandler - Event handling (requires concrete implementation)
// ============================================================================

/// Strategy event handling extension
///
/// Defines how strategy handles various events, requires concrete strategy types to implement
#[async_trait]
pub trait StrategyEventHandlerExt: StrategyMetaDataExt {
    /// Engine event type (e.g., market events, exchange events)
    type EngineEvent: EventTrait;

    /// Node event type (events from nodes)
    type NodeEvent: NodeEventTrait;

    /// Strategy stats event type (performance statistics events)
    // type StrategyStatsEvent: EventTrait;

    /// Handle engine events
    ///
    /// Process events from external engines (market engine, exchange engine, etc.)
    async fn handle_engine_event(&mut self, event: Self::EngineEvent);

    /// Handle node events
    ///
    /// Process events from nodes in the workflow
    /// This is where all node events are aggregated and processed
    async fn handle_node_event(&mut self, node_event: Self::NodeEvent);

    /// Handle strategy command
    ///
    /// Process commands sent to the strategy
    async fn handle_strategy_command(&mut self, command: Self::StrategyCommand);

    // / Handle strategy statistics events
    // /
    // / Process performance statistics and metrics updates
    // async fn handle_strategy_stats_event(&mut self, event: Self::StrategyStatsEvent);
}

// Note: StrategyEventHandler does not provide automatic implementation,
// as it requires concrete strategy types to implement based on business logic

// ============================================================================
// Extension Trait 4: StrategyTaskControl - Task control management
// ============================================================================

/// Strategy task control extension
///
/// Provides task control functionality (cancellation, pause, etc.)
pub trait StrategyTaskControlExt: StrategyMetaDataExt {
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

// Automatically implement StrategyTaskControl for all types that implement StrategyMetaDataExt
impl<Ctx> StrategyTaskControlExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 5: StrategyCommunication - Communication management
// ============================================================================

/// Strategy communication management extension
///
/// Manages communication channels for strategy commands
#[async_trait]
pub trait StrategyCommunicationExt: StrategyMetaDataExt {
    /// Get strategy command sender
    #[inline]
    fn strategy_command_sender(&self) -> &mpsc::Sender<Self::StrategyCommand> {
        self.metadata().strategy_command_sender()
    }

    /// Get strategy command receiver
    #[inline]
    fn strategy_command_receiver(&self) -> &Arc<Mutex<mpsc::Receiver<Self::StrategyCommand>>> {
        &self.metadata().strategy_command_receiver()
    }

    fn add_node_command_sender(&mut self, node_id: NodeId, node_command_sender: mpsc::Sender<Self::NodeCommand>) {
        self.metadata_mut().add_node_command_sender(node_id, node_command_sender);
    }

    fn node_command_sender(&self, node_id: &NodeId) -> &mpsc::Sender<Self::NodeCommand> {
        self.metadata().node_command_sender(node_id)
    }

    async fn send_node_command(&self, command: Self::NodeCommand) -> Result<(), tokio::sync::mpsc::error::SendError<Self::NodeCommand>> {
        let node_id = command.node_id();
        self.node_command_sender(node_id).send(command).await
    }
}

// Automatically implement StrategyCommunication for all types that implement StrategyMetaDataExt
impl<Ctx> StrategyCommunicationExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 6: StrategyStateMachine - State machine operations
// ============================================================================

/// Strategy state machine operations extension
///
/// Manages strategy runtime state and state transitions
#[async_trait]
pub trait StrategyStateMachineExt: StrategyMetaDataExt {
    /// Get state machine reference
    fn state_machine(&self) -> Arc<RwLock<Self::StateMachine>> {
        self.metadata().state_machine()
    }

    /// Get current runtime state
    #[inline]
    async fn run_state(&self) -> <Self::StateMachine as StrategyStateMachine>::State {
        self.state_machine().read().await.current_state().clone()
    }

    /// Check if in specified state
    #[inline]
    async fn is_in_state(&self, state: &<Self::StateMachine as StrategyStateMachine>::State) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// State transition
    #[inline]
    async fn transition_state(
        &self,
        trigger: <Self::StateMachine as StrategyStateMachine>::Trigger,
    ) -> Result<
        StrategyStateChangeActions<
            <Self::StateMachine as StrategyStateMachine>::State,
            <Self::StateMachine as StrategyStateMachine>::Action,
        >,
        StrategyStateMachineError,
    > {
        self.state_machine().write().await.transition(trigger)
    }
}

// Automatically implement StrategyStateMachineOps for all types that implement StrategyMetaDataExt
#[async_trait]
impl<Ctx> StrategyStateMachineExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 7: StrategyVariable - Variable management
// ============================================================================

/// Strategy variable management extension
///
/// Manages strategy variables
#[async_trait]
pub trait StrategyVariableExt: StrategyMetaDataExt {
    async fn init_custom_variables(&mut self, custom_variables: Vec<CustomVariable>) {
        let custom_variable = self.metadata_mut().custom_variable();
        let mut custom_variable_guard = custom_variable.write().await;
        for custom_variable in custom_variables {
            custom_variable_guard.insert(custom_variable.var_name.clone(), custom_variable);
        }
    }

    async fn custom_variable(&self, var_name: &str) -> Result<CustomVariable, StrategyError> {
        let custom_variable = self.metadata().custom_variable();
        let custom_variable_guard = custom_variable.read().await;
        let custom_var = custom_variable_guard.get(var_name).context(CustomVariableNotExistSnafu {
            var_name: var_name.to_string(),
        })?;
        Ok(custom_var.clone())
    }

    async fn strategy_variables(&self) -> Vec<StrategyVariable> {
        let mut strategy_variable = Vec::new();

        let custom_variable = self.metadata().custom_variable();
        let custom_variable_guard = custom_variable.read().await;
        let custom_var = custom_variable_guard
            .iter()
            .map(|(_, custom_variable)| StrategyVariable::CustomVariable(custom_variable.clone()))
            .collect::<Vec<StrategyVariable>>();

        let sys_variable = self.metadata().sys_variable();
        let sys_variable_guard = sys_variable.read().await;
        let sys_var = sys_variable_guard
            .iter()
            .map(|(_, sys_variable)| StrategyVariable::SysVariable(sys_variable.clone()))
            .collect::<Vec<StrategyVariable>>();
        strategy_variable.extend(custom_var);
        strategy_variable.extend(sys_var);
        strategy_variable
    }

    async fn update_sys_variable(&mut self, sys_variable: &SysVariable) {
        let sys = self.metadata_mut().sys_variable();
        let mut sys_variable_guard = sys.write().await;
        sys_variable_guard.insert(sys_variable.var_name.clone(), sys_variable.clone());
    }

    async fn update_custom_variable(
        &mut self,
        var_name: &str,
        operation: &UpdateVarValueOperation,
        operation_value: Option<&VariableValue>,
    ) -> Result<CustomVariable, StrategyError> {
        let custom_variable = self.metadata_mut().custom_variable();
        let mut custom_variable_guard = custom_variable.write().await;

        let custom_var = custom_variable_guard.get_mut(var_name).context(CustomVariableNotExistSnafu {
            var_name: var_name.to_string(),
        })?;

        // 使用工具函数计算新值
        let new_value = apply_variable_operation(&var_name, &custom_var.var_value, operation, operation_value)?;
        // 更新前一个值
        custom_var.previous_value = custom_var.var_value.clone();
        // 更新当前值
        custom_var.var_value = new_value.clone();
        Ok(custom_var.clone())
    }

    async fn reset_custom_variable(&mut self, var_name: &str) -> Result<CustomVariable, StrategyError> {
        let custom_variable = self.metadata_mut().custom_variable();
        let mut custom_variable_guard = custom_variable.write().await;
        let custom_variable = custom_variable_guard.get_mut(var_name).context(CustomVariableNotExistSnafu {
            var_name: var_name.to_string(),
        })?;
        custom_variable.var_value = custom_variable.initial_value.clone();
        Ok(custom_variable.clone())
    }

    async fn reset_all_custom_variables(&mut self) {
        let custom_variable = self.metadata_mut().custom_variable();
        let mut custom_variable_guard = custom_variable.write().await;
        custom_variable_guard.iter_mut().for_each(|(_, custom_variable)| {
            custom_variable.var_value = custom_variable.initial_value.clone();
        });
    }

    async fn reset_all_sys_variables(&mut self) {
        let sys_variable = self.metadata_mut().sys_variable();
        let mut sys_variable_guard = sys_variable.write().await;
        sys_variable_guard.clear();
    }
}

#[async_trait]
impl<Ctx> StrategyVariableExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 8: StrategyPerformance - Performance management
// ============================================================================

/// Strategy performance management extension
///
/// Manages strategy performance
#[async_trait]
pub trait StrategyBenchmarkExt: StrategyMetaDataExt {
    async fn add_node_benchmark(&mut self, node_id: NodeId, node_name: NodeName, node_type: String) {
        self.metadata_mut()
            .benchmark()
            .write()
            .await
            .add_node_benchmark(node_id, node_name, node_type);
    }

    async fn add_node_completed_cycle(&mut self, node_id: NodeId, cycle: CompletedCycle) -> Result<(), StrategyError> {
        self.metadata_mut()
            .benchmark()
            .write()
            .await
            .add_complete_node_cycle(node_id, cycle)?;
        Ok(())
    }

    async fn set_cycle_tracker(&mut self, cycle_tracker: Option<StrategyCycleTracker>) {
        self.metadata_mut().set_cycle_tracker(cycle_tracker).await;
    }

    fn benchmark(&self) -> &Arc<RwLock<StrategyBenchmark>> {
        &self.metadata().benchmark()
    }

    fn cycle_tracker(&self) -> &Arc<RwLock<Option<StrategyCycleTracker>>> {
        &self.metadata().cycle_tracker()
    }

    async fn strategy_performance_report(&self) -> StrategyPerformanceReport {
        let strategy_benchmark = self.metadata().benchmark();
        strategy_benchmark.read().await.report().clone()
    }
}

#[async_trait]
impl<Ctx> StrategyBenchmarkExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Extension Trait 9: StrategyInfra - Infrastructure management
// ============================================================================
pub trait StrategyInfraExt: StrategyMetaDataExt {
    fn database(&self) -> &DatabaseConnection {
        self.metadata().database()
    }

    fn heartbeat(&self) -> &Arc<Mutex<Heartbeat>> {
        self.metadata().heartbeat()
    }
}

#[async_trait]
impl<Ctx> StrategyInfraExt for Ctx where Ctx: StrategyMetaDataExt {}

// ============================================================================
// Composite Trait: StrategyContext (collection of all functionality)
// ============================================================================

/// Complete strategy context trait
///
/// Combines all strategy context required functionality, providing complete capability set for strategies
pub trait StrategyContextExt:
    StrategyMetaDataExt
    + StrategyIdentityExt
    + StrategyWorkflowExt
    + StrategyEventHandlerExt
    + StrategyTaskControlExt
    + StrategyCommunicationExt
    + StrategyStateMachineExt
    + StrategyVariableExt
    + StrategyBenchmarkExt
    + StrategyInfraExt
{
}

// Automatically implement StrategyContext for all types that satisfy all constraints
impl<Ctx> StrategyContextExt for Ctx where
    Ctx: StrategyMetaDataExt
        + StrategyIdentityExt
        + StrategyWorkflowExt
        + StrategyEventHandlerExt
        + StrategyTaskControlExt
        + StrategyCommunicationExt
        + StrategyStateMachineExt
        + StrategyVariableExt
        + StrategyBenchmarkExt
        + StrategyInfraExt
{
}
