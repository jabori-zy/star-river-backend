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
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
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
};

// ============================================================================
// Metadata Trait：NodeMetadata
// ============================================================================

/// 节点上下文核心 trait
///
/// 所有节点上下文必须实现此 trait，提供对基础上下文的访问
pub trait NodeMetaDataExt: Debug + Send + Sync + 'static {
    type StateMachine: StateMachine;
    type NodeEvent: NodeEventTrait + From<CommonEvent>;
    type NodeCommand: NodeCommandTrait;
    type StrategyCommand: StrategyCommandTrait;

    /// 获取基础上下文的不可变引用
    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand>;

    /// 获取基础上下文的可变引用
    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand>;
}

// ============================================================================
// 扩展 Trait 1: NodeIdentity - 节点身份信息（只读）
// ============================================================================

/// 节点身份信息扩展
///
/// 提供节点 ID、名称、类型等只读信息的访问
pub trait NodeInfoExt: NodeMetaDataExt {
    /// 获取周期 ID
    #[inline]
    fn cycle_id(&self) -> CycleId {
        self.metadata().cycle_id()
    }

    /// 获取节点 ID
    #[inline]
    fn node_id(&self) -> &NodeId {
        self.metadata().node_id()
    }

    /// 获取节点名称
    #[inline]
    fn node_name(&self) -> &NodeName {
        self.metadata().node_name()
    }

    /// 获取节点类型
    #[inline]
    fn node_type(&self) -> &NodeType {
        self.metadata().node_type()
    }

    /// 获取策略 ID
    #[inline]
    fn strategy_id(&self) -> StrategyId {
        self.metadata().strategy_id()
    }
}

// 自动为所有实现 NodeMetaDataTrait 的类型实现 NodeIdentity
impl<Ctx> NodeInfoExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// 扩展 Trait 2: NodeRelation - 节点关系管理
// ============================================================================

/// 节点关系管理扩展
///
/// 管理节点之间的拓扑关系（上游节点、叶子节点等）
pub trait NodeRelationExt: NodeMetaDataExt {
    /// 添加源节点（上游节点）ID
    #[inline]
    fn add_source_node(&mut self, source_node_id: NodeId) {
        self.metadata_mut().source_nodes_mut().push(source_node_id);
    }

    /// 检查是否存在指定的源节点
    #[inline]
    fn has_source_node(&self, source_node_id: &str) -> bool {
        self.metadata().source_nodes().iter().any(|id| id == source_node_id)
    }

    /// 获取源节点数量
    #[inline]
    fn source_node_count(&self) -> usize {
        self.metadata().source_nodes().len()
    }

    /// 获取所有源节点 ID
    #[inline]
    fn source_nodes(&self) -> &[NodeId] {
        self.metadata().source_nodes()
    }

    /// 设置是否为叶子节点
    #[inline]
    fn set_leaf_node(&mut self, is_leaf: bool) {
        self.metadata_mut().set_is_leaf_node(is_leaf);
    }

    /// 检查是否为叶子节点
    #[inline]
    fn is_leaf_node(&self) -> bool {
        self.metadata().is_leaf_node()
    }
}

// 自动为所有实现 NodeMetaDataTrait 的类型实现 NodeRelation
impl<Ctx> NodeRelationExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// 扩展 Trait 3: NodeHandle - 句柄管理
// ============================================================================

/// 节点句柄管理扩展
///
/// 管理节点的输入/输出句柄和策略输出句柄
pub trait NodeHandleExt: NodeMetaDataExt + NodeInfoExt {
    fn set_output_handles(&mut self);

    /// 添加输入句柄
    #[inline]
    fn add_input_handle(&mut self, input_handle: NodeInputHandle<Self::NodeEvent>) {
        self.metadata_mut().add_input_handle(input_handle);
    }

    /// 查找输入句柄
    #[inline]
    fn find_input_handle(&self, input_handle_id: &str) -> Option<&NodeInputHandle<Self::NodeEvent>> {
        self.metadata()
            .input_handles()
            .iter()
            .find(|h| h.input_handle_id == input_handle_id)
    }

    /// 获取所有输入句柄
    #[inline]
    fn input_handles(&self) -> &[NodeInputHandle<Self::NodeEvent>] {
        self.metadata().input_handles()
    }

    // ------------------------------------------------------------------------
    // 输出句柄管理
    // ------------------------------------------------------------------------

    #[inline]
    fn add_default_output_handle(&mut self, output_handle: NodeOutputHandle<Self::NodeEvent>) {
        self.metadata_mut().add_output_handle(output_handle);
    }

    /// 获取默认输出句柄
    #[inline]
    fn default_output_handle(&self) -> Result<&NodeOutputHandle<Self::NodeEvent>, NodeError> {
        let default_handle_id = generate_default_output_handle_id(self.node_id());
        self.metadata()
            .output_handles()
            .get(&default_handle_id)
            .context(OutputHandleNotFoundSnafu {
                handle_id: default_handle_id.to_string(),
            })
    }

    /// 检查是否有默认输出句柄
    #[inline]
    fn has_default_output_handle(&self) -> bool {
        let default_handle_id = format!("{}:default", self.node_id());
        self.metadata().output_handles().contains_key(&default_handle_id)
    }

    /// 添加输出句柄
    #[inline]
    fn add_output_handle(&mut self, is_default: bool, config_id: i32, handle_id: HandleId, sender: broadcast::Sender<Self::NodeEvent>) {
        let node_id = self.node_id().clone();
        let handle = NodeOutputHandle::new(node_id, is_default, config_id, handle_id, sender);
        self.metadata_mut().add_output_handle(handle);
    }

    #[inline]
    fn output_handles(&self) -> &HashMap<HandleId, NodeOutputHandle<Self::NodeEvent>> {
        self.metadata().output_handles()
    }

    /// 获取输出句柄
    #[inline]
    fn output_handle(&self, handle_id: &str) -> Option<&NodeOutputHandle<Self::NodeEvent>> {
        self.metadata().output_handles().get(handle_id)
    }

    /// 检查输出句柄是否存在
    #[inline]
    fn has_output_handle(&self, handle_id: &str) -> bool {
        self.metadata().output_handles().contains_key(handle_id)
    }

    /// 获取策略输出句柄 (strategy bound handle)
    #[inline]
    fn strategy_bound_handle(&self) -> &NodeOutputHandle<Self::NodeEvent> {
        self.metadata().strategy_bound_handle()
    }

    #[inline]
    fn subscribe_strategy_bound_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<Self::NodeEvent> {
        self.metadata_mut().subscribe_strategy_bound_handle(subscriber_id)
    }

    fn subscribe_output_handle(&mut self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<Self::NodeEvent> {
        self.metadata_mut().subscribe_output_handle(handle_id, subscriber_id)
    }
}

// ============================================================================
// 扩展 Trait 4: NodeStateMachineOps - 状态机操作
// ============================================================================

/// 节点状态机操作扩展
///
/// 管理节点的运行状态和状态转换
#[async_trait]
pub trait NodeStateMachineExt: NodeMetaDataExt {
    /// 获取状态机引用
    fn state_machine(&self) -> Arc<RwLock<Self::StateMachine>> {
        self.metadata().state_machine()
    }

    /// 获取当前运行状态
    #[inline]
    async fn run_state(&self) -> <Self::StateMachine as StateMachine>::State {
        self.state_machine().read().await.current_state().clone()
    }

    /// 检查是否处于指定状态
    #[inline]
    async fn is_in_state(&self, state: &<Self::StateMachine as StateMachine>::State) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// 状态转换
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

// 自动为所有实现 NodeMetaDataTrait 的类型实现 NodeStateMachineOps
#[async_trait]
impl<Ctx> NodeStateMachineExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// 扩展 Trait 5: NodeCommunication - 通信管理
// ============================================================================

/// 节点通信管理扩展
///
/// 管理与策略和其他节点的通信，包括命令收发和事件发送
#[async_trait]
pub trait NodeCommunicationExt: NodeMetaDataExt + NodeInfoExt + NodeRelationExt + NodeHandleExt {
    /// 获取策略命令发送器
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

    /// 获取节点命令接收器
    #[inline]
    fn node_command_receiver(&self) -> Arc<Mutex<mpsc::Receiver<Self::NodeCommand>>> {
        self.metadata().node_command_receiver()
    }

    /// 发送事件到指定的输出句柄
    ///
    /// # Arguments
    /// - `handle_id` - 输出句柄 ID
    /// - `event` - 要发送的事件
    fn output_handle_send(&self, event: Self::NodeEvent) -> Result<(), crate::error::NodeError> {
        let handle_id = event.output_handle_id().to_string();
        let output_handle = self.output_handle(&handle_id).context(OutputHandleNotFoundSnafu {
            handle_id: handle_id.clone(),
        })?;

        if output_handle.is_connected() {
            output_handle
                .send(event)
                .map_err(|e| NodeEventSendFailedSnafu { handle_id: handle_id }.into_error(Arc::new(e)))?;
        } else {
            // tracing::warn!(
            //     "@[{}] output handle {} is not connected, skip sending event",
            //     self.node_name(),
            //     handle_id
            // );
        }

        Ok(())
    }

    /// 发送事件到策略绑定句柄
    ///
    /// # Arguments
    /// - `event` - 要发送的事件
    fn strategy_bound_handle_send(&self, event: Self::NodeEvent) -> Result<(), NodeError> {
        let strategy_handle = self.strategy_bound_handle();

        strategy_handle.send(event)
    }

    /// 发送事件到默认输出句柄
    ///
    /// # Arguments
    /// - `event` - 要发送的事件
    fn default_output_handle_send(&self, event: Self::NodeEvent) -> Result<(), crate::error::NodeError> {
        let default_handle = self.default_output_handle()?;

        if default_handle.is_connected() {
            default_handle.send(event)
        } else {
            Ok(())
        }
    }

    fn send_execute_over_event(&self, cycle_id: CycleId, config_id: Option<i32>) -> Result<(), NodeError> {
        if !self.is_leaf_node() {
            return Ok(());
        }

        let payload = ExecuteOverPayload::new(cycle_id, config_id);
        let execute_over_event: CommonEvent = ExecuteOverEvent::new(
            self.node_id().clone(),
            self.node_name().to_string(),
            self.strategy_bound_handle().output_handle_id().clone(),
            payload,
        )
        .into();

        self.strategy_bound_handle_send(execute_over_event.into())?;

        Ok(())
    }

    // send trigger event to downstream node. if current node is leaf node, send execute over event instead.
    async fn send_trigger_event(&self, handle_id: &str) -> Result<(), NodeError> {
        // 叶子节点不发送触发事件
        if self.is_leaf_node() {
            // self.send_execute_over_event()?;
            return Ok(());
        }

        let payload = TriggerPayload::new(self.cycle_id());
        let trigger_event: CommonEvent =
            TriggerEvent::new(self.node_id().clone(), self.node_name().to_string(), handle_id.to_string(), payload).into();

        let output_handle = self.output_handle(handle_id).context(OutputHandleNotFoundSnafu {
            handle_id: handle_id.to_string(),
        })?;

        if output_handle.is_connected() {
            output_handle.send(trigger_event.into())
        } else {
            // tracing::warn!("@[{}] output handle {} is not connected, skip sending event", self.node_name(), handle_id);
            Ok(())
        }
    }
}

// 自动为所有实现 NodeMetaDataTrait 的类型实现 NodeControl
impl<Ctx> NodeCommunicationExt for Ctx where Ctx: NodeMetaDataExt + NodeInfoExt + NodeRelationExt + NodeHandleExt {}

// ============================================================================
// 扩展 Trait 6: NodeControl - 节点运行控制
// ============================================================================

/// 节点运行控制扩展
///
/// 提供节点运行控制相关功能（取消、暂停等）
pub trait NodeTaskControlExt: NodeMetaDataExt {
    /// 获取取消令牌
    #[inline]
    fn cancel_token(&self) -> &CancellationToken {
        self.metadata().cancel_token()
    }

    /// 检查是否已取消
    #[inline]
    fn is_cancelled(&self) -> bool {
        self.cancel_token().is_cancelled()
    }

    /// 请求取消
    #[inline]
    fn request_cancel(&self) {
        self.cancel_token().cancel();
    }
}

// 自动为所有实现 NodeMetaDataTrait 的类型实现 NodeControl
impl<Ctx> NodeTaskControlExt for Ctx where Ctx: NodeMetaDataExt {}

// ============================================================================
// 扩展 Trait 7: NodeEventHandler - 事件处理（需要具体实现）
// ============================================================================

/// 节点事件处理扩展
///
/// 定义节点如何处理各种事件，需要具体节点类型实现
#[async_trait]
pub trait NodeEventHandlerExt: NodeMetaDataExt {
    type EngineEvent: EventTrait;
    type Error: StarRiverErrorTrait;

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) -> Result<(), Self::Error>;
    /// 处理节点事件
    ///
    /// 默认实现仅记录日志，具体节点应该覆盖此方法
    async fn handle_source_node_event(&mut self, node_event: Self::NodeEvent) -> Result<(), Self::Error>;

    /// 处理节点命令
    ///
    /// 默认实现仅记录日志，具体节点应该覆盖此方法
    async fn handle_command(&mut self, node_command: Self::NodeCommand) -> Result<(), Self::Error>;
}

// 注意：NodeEventHandler 不提供自动实现，因为它需要具体节点类型根据业务逻辑来实现

// ============================================================================
// 扩展 Trait 8: NodeBenchmark - 性能统计（提供默认实现）
// ============================================================================

/// 节点性能统计扩展
///
/// 提供向策略发送性能统计数据的功能
#[async_trait]
pub trait NodeBenchmarkExt: NodeMetaDataExt + NodeInfoExt + NodeCommunicationExt {
    type Error: StarRiverErrorTrait;

    /// 挂载节点周期追踪数据
    ///
    /// 将节点的性能统计数据发送到策略层
    ///
    /// # Arguments
    /// - `node_id` - 节点 ID
    /// - `node_name` - 节点名称
    /// - `cycle_tracker` - 周期追踪数据
    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error>;
}

// ============================================================================
// 组合 Trait：StrategyNodeContext（所有功能的集合）
// ============================================================================

/// 策略节点上下文完整 trait
///
/// 组合了所有节点上下文需要的功能，为节点提供完整的能力集
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

// 自动为所有满足所有约束的类型实现 StrategyNodeContext
impl<Ctx> NodeContextExt for Ctx where
    Ctx: NodeMetaDataExt
        + NodeInfoExt
        + NodeRelationExt
        + NodeHandleExt
        + NodeStateMachineExt
        + NodeCommunicationExt
        + NodeTaskControlExt
        + NodeEventHandlerExt
        + NodeBenchmarkExt // Ctx::Event: Clone + Send + Sync,
                           // Ctx::NodeCommand: Send,
{
}
