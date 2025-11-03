// ============================================================================
// 模块声明
// ============================================================================
pub(crate) mod base_context;
pub(crate) mod node_context_trait;
pub(crate) mod node_handles;
pub(crate) mod node_state_machine;
pub(crate) mod node_trait;
pub(crate) mod node_utils;
pub(crate) mod node_message;

pub use node_state_machine::NodeRunState;


// std
use std::{
    collections::HashMap,
    fmt::Debug,
    str::FromStr,
    sync::{Arc, LazyLock},
};

// third-party
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use tokio::sync::{RwLock, broadcast};
use tokio_util::sync::CancellationToken;

// workspace crate
use event_center::{Channel, event::node_event::BacktestNodeEvent};
use star_river_core::custom_type::{NodeId, NodeName};

// current crate
use node_context_trait::{NodeBaseContextTrait, NodeControl, NodeHandleTrait, NodeIdentity, NodeRelation, NodeStateMachineTrait};
use node_handles::NodeInputHandle;
use node_trait::{NodeContextAccessor, NodeLifecycle};
use crate::{
    error::node_error::BacktestNodeError,
    node_list::{KlineNode, StartNode}
};

#[derive(Debug)]
pub struct NodeBase<C, Action>
where
    C: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
    pub context: Arc<RwLock<C>>,
    _phantom: std::marker::PhantomData<Action>,
}

impl<C, Action> NodeBase<C, Action>
where
    C: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
    /// 创建新的节点基础实例
    pub fn new(context: C) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C, Action> Clone for NodeBase<C, Action>
where
    C: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + 'static,
{
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context), // 克隆 Arc（引用计数+1），共享底层数据
            _phantom: std::marker::PhantomData,
        }
    }
}

// 为 NodeBase 实现 NodeContextAccessor trait
impl<C, Action> NodeContextAccessor<C, Action> for NodeBase<C, Action>
where
    C: NodeBaseContextTrait<Action>,
    Action: Clone + Debug + Send + Sync + 'static,
{
    fn context(&self) -> &Arc<RwLock<C>> {
        &self.context
    }
}

// ============================================================================
// BacktestNode 枚举
// ============================================================================

#[derive(Debug, Clone)]
pub enum BacktestNode {
    Start(StartNode),
    Kline(KlineNode),
}

// 为 BacktestNode 实现 From trait
impl From<StartNode> for BacktestNode {
    fn from(node: StartNode) -> Self {
        BacktestNode::Start(node)
    }
}

impl From<KlineNode> for BacktestNode {
    fn from(node: KlineNode) -> Self {
        BacktestNode::Kline(node)
    }
}








impl BacktestNode {
    pub fn klind(&self) -> NodeType {
        match self {
            BacktestNode::Start(_) => NodeType::StartNode,
            BacktestNode::Kline(_) => NodeType::KlineNode,
        }
    }

    pub async fn node_id(&self) -> NodeId {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
        }
    }

    pub async fn node_name(&self) -> NodeName {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
        }
    }

    pub async fn node_type(&self) -> NodeType {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
        }
    }

    pub async fn is_in_state(&self, state: NodeRunState) -> bool {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await })).await,
            BacktestNode::Kline(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await })).await,
        }
    }

    pub async fn run_state(&self) -> NodeRunState {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::Kline(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
        }
    }

    pub async fn cancel_token(&self) -> CancellationToken {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
        }
    }

    pub async fn subscribe_strategy_output_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_output_handle(subscriber_id)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_output_handle(subscriber_id)).await,
        }
    }

    pub async fn subscribe_output_handle(&self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        match self {
            BacktestNode::Start(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            BacktestNode::Kline(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
        }
    }

    pub async fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
        }
    }

    pub async fn set_output_handles(&mut self) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
        }
    }

    pub async fn set_leaf_node(&mut self, is_leaf_node: bool) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
        }
    }

    pub async fn add_source_node(&mut self, source_node_id: String) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
        }
    }

    pub async fn init(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.init().await,
            BacktestNode::Kline(node) => node.init().await,
        }
    }

    pub async fn stop(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.stop().await,
            BacktestNode::Kline(node) => node.stop().await,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeType {
    StartNode,
    KlineNode,
    IndicatorNode,
    IfElseNode,
    FuturesOrderNode,
    PositionNode,
    VariableNode,
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 处理指标节点的特殊情况
        if s.ends_with("indicator_node") || s.ends_with("IndicatorNode") {
            return Ok(NodeType::IndicatorNode);
        }

        // 支持驼峰和下划线两种命名方式
        match s {
            // 下划线格式
            "start_node" => Ok(NodeType::StartNode),
            "kline_node" => Ok(NodeType::KlineNode),
            "indicator_node" => Ok(NodeType::IndicatorNode),
            "if_else_node" => Ok(NodeType::IfElseNode),
            "futures_order_node" => Ok(NodeType::FuturesOrderNode),
            "position_node" => Ok(NodeType::PositionNode),
            "position_management_node" => Ok(NodeType::PositionNode),
            "variable_node" => Ok(NodeType::VariableNode),
            // 驼峰格式
            "startNode" => Ok(NodeType::StartNode),
            "klineNode" => Ok(NodeType::KlineNode),
            "IndicatorNode" => Ok(NodeType::IndicatorNode),
            "ifElseNode" => Ok(NodeType::IfElseNode),
            "futuresOrderNode" => Ok(NodeType::FuturesOrderNode),
            "positionNode" => Ok(NodeType::PositionNode),
            "positionManagementNode" => Ok(NodeType::PositionNode),
            "variableNode" => Ok(NodeType::VariableNode),
            _ => Err(format!("Unknown node type: {}", s)),
        }
    }
}

static BACKTEST_NODE_EVENT_RECEIVERS: LazyLock<HashMap<NodeType, Vec<Channel>>> = LazyLock::new(|| {
    HashMap::from([
        (NodeType::StartNode, vec![]),
        (NodeType::KlineNode, vec![Channel::Market]),
        (NodeType::IndicatorNode, vec![]),
        (NodeType::IfElseNode, vec![]),
        (NodeType::FuturesOrderNode, vec![]),
        (NodeType::PositionNode, vec![]),
        (NodeType::VariableNode, vec![]),
        (NodeType::VariableNode, vec![]),
    ])
});

pub struct BacktestNodeEventReceiver;

impl BacktestNodeEventReceiver {
    pub fn get_external_event_receivers(node_kind: &NodeType) -> Vec<Channel> {
        BACKTEST_NODE_EVENT_RECEIVERS.get(node_kind).cloned().unwrap_or_default()
    }
}
