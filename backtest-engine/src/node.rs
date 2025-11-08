// ============================================================================
// 模块声明
// ============================================================================
// pub(crate) mod base_context;
// pub(crate) mod node_context_trait;
// pub(crate) mod node_handles;
// pub(crate) mod node_state_machine;
// pub(crate) mod node_trait;
// pub(crate) mod node_utils;
// pub(crate) mod node_message;

// pub use node_state_machine::NodeRunState;


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
use event_center_new::event::Channel;
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::node::{NodeType,NodeTrait};

// current crate
use node_context_trait::{NodeBaseContextTrait, NodeControl, NodeHandleTrait, NodeIdentity, NodeRelation, NodeStateMachineTrait};
use node_handles::NodeInputHandle;
use node_trait::{NodeContextAccessor, NodeLifecycle};
use crate::{
    error::node_error::BacktestNodeError,
    node_event::BacktestNodeEvent
};

use crate::node_list_new::start_node::StartNode;

// ============================================================================
// BacktestNode 枚举
// ============================================================================

#[derive(Debug, Clone)]
pub enum BacktestNode {
    Start(StartNode),
    // Kline(KlineNode),
}

// 为 BacktestNode 实现 From trait
impl From<StartNode> for BacktestNode {
    fn from(node: StartNode) -> Self {
        BacktestNode::Start(node)
    }
}

// impl From<KlineNode> for BacktestNode {
//     fn from(node: KlineNode) -> Self {
//         BacktestNode::Kline(node)
//     }
// }


impl NodeTrait for BacktestNode {}





impl BacktestNode {
    pub fn klind(&self) -> NodeType {
        match self {
            BacktestNode::Start(_) => NodeType::StartNode,
            // BacktestNode::Kline(_) => NodeType::KlineNode,
        }
    }

    pub async fn node_id(&self) -> NodeId {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            // BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
        }
    }

    pub async fn node_name(&self) -> NodeName {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            // BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
        }
    }

    pub async fn node_type(&self) -> NodeType {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            // BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
        }
    }

    pub async fn is_in_state(&self, state: NodeRunState) -> bool {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await })).await,
            // BacktestNode::Kline(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await })).await,
        }
    }

    pub async fn run_state(&self) -> NodeRunState {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            // BacktestNode::Kline(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
        }
    }

    pub async fn cancel_token(&self) -> CancellationToken {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            // BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
        }
    }

    pub async fn subscribe_strategy_output_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_output_handle(subscriber_id)).await,
            // BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_output_handle(subscriber_id)).await,
        }
    }

    pub async fn subscribe_output_handle(&self, handle_id: String, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        match self {
            BacktestNode::Start(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            // BacktestNode::Kline(node) => {
            //     node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
            //         .await
            // }
        }
    }

    pub async fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            // BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
        }
    }

    pub async fn set_output_handles(&mut self) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            // BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
        }
    }

    pub async fn set_leaf_node(&mut self, is_leaf_node: bool) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            // BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
        }
    }

    pub async fn add_source_node(&mut self, source_node_id: String) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            // BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
        }
    }

    pub async fn init(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.init().await,
            // BacktestNode::Kline(node) => node.init().await,
        }
    }

    pub async fn stop(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.stop().await,
            // BacktestNode::Kline(node) => node.stop().await,
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
