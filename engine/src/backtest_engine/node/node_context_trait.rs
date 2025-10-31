// ============================================================================
// 标准库导入
// ============================================================================

use std::fmt::Debug;
use std::sync::Arc;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use async_trait::async_trait;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeCommandReceiver, StrategyCommandSender};
use event_center::event::Event;
use event_center::event::node_event::BacktestNodeEvent;
use star_river_core::custom_type::{HandleId, NodeId, NodeName, PlayIndex};
use star_river_core::error::node_error::node_state_machine_error::BacktestNodeStateMachineError;
use star_river_core::strategy::node_benchmark::CompletedCycle;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::backtest_engine::node::node_state_machine::NodeStateMachine;
use super::node_utils::NodeUtils;

// ============================================================================
// 当前模块内部导入
// ============================================================================
use super::NodeType;
use super::base_context::NodeBaseContext;
use super::node_handles::{NodeInputHandle, NodeOutputHandle};
use super::node_state_machine::{NodeRunState, NodeStateTransTrigger, StateChangeActions};

// ============================================================================
// 核心 Trait：NodeContext
// ============================================================================

/// 节点上下文核心 trait
///
/// 所有节点上下文必须实现此 trait，提供对基础上下文的访问
pub trait NodeBaseContextTrait<Action>: Debug + Send + Sync + Clone + 'static
where
    Action: Clone + Debug + 'static,
{
    /// 获取基础上下文的不可变引用
    fn base_context(&self) -> &NodeBaseContext<Action>;

    /// 获取基础上下文的可变引用
    fn base_context_mut(&mut self) -> &mut NodeBaseContext<Action>;
}

// ============================================================================
// 扩展 Trait 1: NodeIdentity - 节点身份信息（只读）
// ============================================================================

/// 节点身份信息扩展
///
/// 提供节点 ID、名称、类型等只读信息的访问
pub trait NodeIdentity<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 获取节点 ID
    #[inline]
    fn node_id(&self) -> &NodeId {
        self.base_context().node_id()
    }

    /// 获取节点名称
    #[inline]
    fn node_name(&self) -> &NodeName{
        self.base_context().node_name()
    }

    /// 获取节点类型
    #[inline]
    fn node_type(&self) -> &NodeType {
        self.base_context().node_type()
    }

    /// 获取策略 ID
    #[inline]
    fn strategy_id(&self) -> i32 {
        self.base_context().strategy_id()
    }
}

// 自动为所有实现 NodeContextTrait 的类型实现 NodeIdentity
impl<T, Action> NodeIdentity<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 2: NodeRelation - 节点关系管理
// ============================================================================

/// 节点关系管理扩展
///
/// 管理节点之间的拓扑关系（上游节点、叶子节点等）
pub trait NodeRelation<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 添加源节点（上游节点）ID
    #[inline]
    fn add_source_node(&mut self, source_node_id: NodeId) {
        self.base_context_mut().source_nodes_mut().push(source_node_id);
    }

    /// 检查是否存在指定的源节点
    #[inline]
    fn has_source_node(&self, source_node_id: &str) -> bool {
        self.base_context().source_nodes().iter().any(|id| id == source_node_id)
    }

    /// 获取源节点数量
    #[inline]
    fn source_node_count(&self) -> usize {
        self.base_context().source_nodes().len()
    }

    /// 获取所有源节点 ID
    #[inline]
    fn source_nodes(&self) -> &[NodeId] {
        self.base_context().source_nodes()
    }

    /// 设置是否为叶子节点
    #[inline]
    fn set_leaf_node(&mut self, is_leaf: bool) {
        self.base_context_mut().set_is_leaf_node(is_leaf);
    }

    /// 检查是否为叶子节点
    #[inline]
    fn is_leaf_node(&self) -> bool {
        self.base_context().is_leaf_node()
    }
}

// 自动为所有实现 NodeContextTrait 的类型实现 NodeRelation
impl<T, Action> NodeRelation<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 3: NodeHandle - 句柄管理
// ============================================================================

/// 节点句柄管理扩展
///
/// 管理节点的输入/输出句柄和策略输出句柄
pub trait NodeHandleTrait<Action>: NodeBaseContextTrait<Action> + NodeIdentity<Action>
where
    Action: Clone + Debug + 'static,
{
    // ------------------------------------------------------------------------
    // 输入句柄管理
    // ------------------------------------------------------------------------

    fn set_output_handles(&mut self);

    /// 添加输入句柄
    #[inline]
    fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        self.base_context_mut().add_input_handle(input_handle);
    }

    /// 查找输入句柄
    #[inline]
    fn find_input_handle(&self, input_handle_id: &str) -> Option<&NodeInputHandle> {
        self.base_context()
            .input_handles()
            .iter()
            .find(|h| h.input_handle_id == input_handle_id)
    }

    /// 获取所有输入句柄
    #[inline]
    fn input_handles(&self) -> &[NodeInputHandle] {
        self.base_context().input_handles()
    }

    // ------------------------------------------------------------------------
    // 输出句柄管理
    // ------------------------------------------------------------------------

    /// 获取默认输出句柄
    #[inline]
    fn default_output_handle(&self) -> Option<&NodeOutputHandle> {
        let default_handle_id = NodeUtils::generate_default_output_handle_id(self.node_id());
        self.base_context().output_handles().get(&default_handle_id)
    }

    /// 检查是否有默认输出句柄
    #[inline]
    fn has_default_output_handle(&self) -> bool {
        let default_handle_id = format!("{}:default", self.node_id());
        self.base_context().output_handles().contains_key(&default_handle_id)
    }

    /// 添加输出句柄
    #[inline]
    fn add_output_handle(&mut self, handle_id: HandleId, sender: broadcast::Sender<BacktestNodeEvent>) {
        let node_id = self.node_id().clone();
        let handle = NodeOutputHandle::new(node_id, handle_id.clone(), sender);
        self.base_context_mut().add_output_handle(handle);
    }

    /// 获取输出句柄
    #[inline]
    fn output_handle(&self, handle_id: &str) -> Option<&NodeOutputHandle> {
        self.base_context().output_handles().get(handle_id)
    }

    /// 检查输出句柄是否存在
    #[inline]
    fn has_output_handle(&self, handle_id: &str) -> bool {
        self.base_context().output_handles().contains_key(handle_id)
    }

    /// 获取策略输出句柄
    #[inline]
    fn strategy_output_handle(&self) -> &NodeOutputHandle {
        self.base_context().strategy_output_handle()
    }

    #[inline]
    fn subscribe_strategy_output_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        self.base_context_mut().subscribe_strategy_output_handle(subscriber_id)
    }

    fn subscribe_output_handle(&mut self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        self.base_context_mut().subscribe_output_handle(handle_id, subscriber_id)
    }
}

// 自动为所有实现 NodeContextTrait + NodeIdentity 的类型实现 NodeHandle
// impl<T, Action> NodeHandleTrait<Action> for T
// where
//     T: NodeBaseContextTrait<Action> + NodeIdentity<Action>,
//     Action: Clone + Debug + 'static,
// {
// }

// ============================================================================
// 扩展 Trait 4: NodeStateMachine - 状态机管理
// ============================================================================

/// 节点状态机管理扩展
///
/// 管理节点的运行状态和状态转换
pub trait NodeStateMachineTrait<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{

    fn state_machine(&self) -> Arc<RwLock<NodeStateMachine<Action>>> {
        self.base_context().state_machine()
    }


    /// 获取当前运行状态
    #[inline]
    async fn run_state(&self) -> NodeRunState {
        self.state_machine().read().await.current_state().clone()
    }

    /// 检查是否处于指定状态
    #[inline]
    async fn is_in_state(&self, state: &NodeRunState) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// 状态转换
    #[inline]
    async fn transition_state(&self, event: NodeStateTransTrigger) -> Result<StateChangeActions<Action>, BacktestNodeStateMachineError> {
        self.state_machine().write().await.transition(event)
    }
}

// 自动为所有实现 NodeContextTrait 的类型实现 NodeStateMachine
impl<T, Action> NodeStateMachineTrait<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 5: NodePlayback - 播放控制
// ============================================================================

/// 节点播放控制扩展
///
/// 管理回测播放进度相关功能
pub trait NodePlayback<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 获取当前播放索引
    #[inline]
    fn play_index(&self) -> PlayIndex {
        self.base_context().play_index()
    }

    /// 订阅播放索引变化
    #[inline]
    fn subscribe_play_index(&self) -> tokio::sync::watch::Receiver<PlayIndex> {
        self.base_context().play_index_watch_rx().clone()
    }

    /// 获取播放索引接收器引用
    #[inline]
    fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        self.base_context().play_index_watch_rx()
    }
}

// 自动为所有实现 NodeContextTrait 的类型实现 NodePlayback
impl<T, Action> NodePlayback<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 6: NodeCommunication - 通信管理
// ============================================================================

/// 节点通信管理扩展
///
/// 管理与策略和其他节点的通信
pub trait NodeCommunication<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 获取策略命令发送器
    #[inline]
    fn strategy_command_sender(&self) -> &StrategyCommandSender {
        self.base_context().strategy_command_sender()
    }

    /// 获取节点命令接收器
    #[inline]
    fn node_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.base_context().node_command_receiver()
    }
}

// 自动为所有实现 NodeContextTrait 的类型实现 NodeCommunication
impl<T, Action> NodeCommunication<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 7: NodeControl - 节点运行控制
// ============================================================================

/// 节点运行控制扩展
///
/// 提供节点运行控制相关功能（取消、暂停等）
pub trait NodeControl<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 获取取消令牌
    #[inline]
    fn cancel_token(&self) -> &CancellationToken {
        self.base_context().cancel_token()
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

// 自动为所有实现 NodeBaseContextTrait 的类型实现 NodeControl
impl<T, Action> NodeControl<Action> for T
where
    T: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 8: NodeEventHandler - 事件处理（需要具体实现）
// ============================================================================

/// 节点事件处理扩展
///
/// 定义节点如何处理各种事件，需要具体节点类型实现
#[async_trait]
pub trait NodeEventHandler<Action>: NodeBaseContextTrait<Action>
where
    Action: Clone + Debug + 'static,
{
    /// 处理引擎事件
    async fn handle_engine_event(&mut self, event: Event) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }

    /// 处理节点事件
    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        tracing::info!("[{}] received node event: {:?}", self.node_name(), node_event);
    }

    /// 处理节点命令
    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        tracing::info!("[{}] received node command: {:?}", self.node_name(), node_command);
    }
}

// ============================================================================
// 扩展 Trait 9: NodeEventEmitter - 事件发送（提供默认实现）
// ============================================================================

/// 节点事件发送扩展
///
/// 提供向下游节点和策略发送事件的功能
#[async_trait]
pub trait NodeEventEmitter<Action>:
    NodeBaseContextTrait<Action> + NodeIdentity<Action> + NodeRelation<Action> + NodePlayback<Action> + NodeHandleTrait<Action> + Send + Sync
where
    Action: Clone + Debug + 'static,
{
    /// 发送执行完成事件
    ///
    /// 仅叶子节点会发送此事件
    async fn send_execute_over_event(&self) -> Result<(), String> {
        use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload};

        // 非叶子节点不发送执行结束事件
        if !self.is_leaf_node() {
            return Ok(());
        }

        let payload = ExecuteOverPayload::new(self.play_index());
        let execute_over_event: CommonEvent = ExecuteOverEvent::new(
            self.node_id().clone(),
            self.node_name().to_string(),
            self.node_id().clone(),
            payload,
        )
        .into();

        let strategy_output_handle = self.strategy_output_handle();
        strategy_output_handle
            .send(execute_over_event.into())
            .map_err(|e| format!("Failed to send execute over event: {}", e))?;

        Ok(())
    }

    /// 发送触发事件
    ///
    /// 向指定输出句柄发送触发事件，叶子节点不发送
    async fn send_trigger_event(&self, handle_id: &str) -> Result<(), String> {
        use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};

        // 叶子节点不发送触发事件
        if self.is_leaf_node() {
            return Ok(());
        }

        let payload = TriggerPayload::new(self.play_index());
        let trigger_event: CommonEvent =
            TriggerEvent::new(self.node_id().clone(), self.node_name().to_string(), handle_id.to_string(), payload).into();

        let output_handle = self
            .output_handle(handle_id)
            .ok_or_else(|| format!("Output handle not found: {}", handle_id))?;

        output_handle
            .send(trigger_event.into())
            .map_err(|e| format!("Failed to send trigger event: {}", e))?;

        Ok(())
    }
}

// 自动为所有满足约束的类型实现 NodeEventEmitter
impl<T, Action> NodeEventEmitter<Action> for T
where
    T: NodeBaseContextTrait<Action>
        + NodeIdentity<Action>
        + NodeRelation<Action>
        + NodePlayback<Action>
        + NodeHandleTrait<Action>
        + Send
        + Sync,
    Action: Clone + Debug + 'static,
{
}

// ============================================================================
// 扩展 Trait 10: NodeBenchmark - 性能统计
// ============================================================================

/// 节点性能统计扩展
///
/// 提供节点性能数据收集功能
#[async_trait]
pub trait NodeBenchmark<Action>: NodeBaseContextTrait<Action> + NodeIdentity<Action> + NodeCommunication<Action> + Send + Sync
where
    Action: Clone + Debug + 'static,
{
    /// 添加节点周期追踪数据
    async fn add_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), String> {
        use event_center::communication::backtest_strategy::{AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand};

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let payload = AddNodeCycleTrackerCmdPayload::new(node_id.clone(), cycle_tracker);
        let command = AddNodeCycleTrackerCommand::new(node_id, resp_tx, Some(payload)).into();

        self.strategy_command_sender()
            .send(command)
            .await
            .map_err(|e| format!("Failed to send add cycle tracker command: {}", e))?;

        resp_rx
            .await
            .map_err(|e| format!("Failed to receive add cycle tracker response: {}", e))?;

        Ok(())
    }
}

// 自动为所有满足约束的类型实现 NodeBenchmark
impl<T, Action> NodeBenchmark<Action> for T
where
    T: NodeBaseContextTrait<Action> + NodeIdentity<Action> + NodeCommunication<Action> + Send + Sync,
    Action: Clone + Debug + 'static,
{
}


// ============================================================================
// 组合 Trait：BacktestNodeContext（所有功能的集合）
// ============================================================================

/// 回测节点上下文完整 trait
///
/// 组合了所有节点上下文需要的功能
pub trait BacktestNodeContext<Action>:
    NodeBaseContextTrait<Action>
    + NodeIdentity<Action>
    + NodeRelation<Action>
    + NodeHandleTrait<Action>
    + NodeStateMachineTrait<Action>
    + NodePlayback<Action>
    + NodeCommunication<Action>
    + NodeControl<Action>
    + NodeEventHandler<Action>
    + NodeEventEmitter<Action>
    + NodeBenchmark<Action>
    // + NodeLifecycle<Action>
where
    Action: Clone + Debug + 'static,
{
}

// 自动为所有满足所有约束的类型实现 BacktestNodeContext
impl<T, Action> BacktestNodeContext<Action> for T
where
    T: NodeBaseContextTrait<Action>
        + NodeIdentity<Action>
        + NodeRelation<Action>
        + NodeHandleTrait<Action>
        + NodeStateMachineTrait<Action>
        + NodePlayback<Action>
        + NodeCommunication<Action>
        + NodeControl<Action>
        + NodeEventHandler<Action>
        + NodeEventEmitter<Action>
        + NodeBenchmark<Action>,
        // + NodeLifecycle<Action>,
    Action: Clone + Debug + 'static,
{
}
