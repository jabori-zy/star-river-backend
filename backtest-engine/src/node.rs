pub(crate) mod node_command;
pub(crate) mod node_error;
pub(crate) mod node_event;
pub(crate) mod node_message;
pub(crate) mod node_state_machine;
pub(crate) mod node_utils;

// Standard library imports
use std::{collections::HashMap, fmt::Debug, sync::LazyLock};

// External crate imports
use async_trait::async_trait;
use derive_more::From;
// Workspace crate imports
use event_center::event::Channel;
use node_error::BacktestNodeError;
use node_event::BacktestNodeEvent;
use star_river_core::custom_type::{HandleId, NodeId, NodeName};
use strategy_core::node::{
    NodeTrait, NodeType,
    context_trait::{NodeHandleExt, NodeIdentityExt, NodeRelationExt, NodeStateMachineExt, NodeTaskControlExt},
    node_handles::{NodeInputHandle, NodeOutputHandle},
    node_trait::{NodeContextAccessor, NodeLifecycle},
};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

// Current crate imports
use crate::{
    node::node_state_machine::NodeRunState,
    node_catalog::{
        futures_order_node::FuturesOrderNode, if_else_node::IfElseNode, indicator_node::IndicatorNode, kline_node::KlineNode,
        position_node::PositionNode, start_node::StartNode, variable_node::VariableNode,
    },
};

#[derive(Debug, Clone, From)]
pub enum BacktestNode {
    Start(StartNode),
    Kline(KlineNode),
    Indicator(IndicatorNode),
    IfElse(IfElseNode),
    FuturesOrder(FuturesOrderNode),
    Position(PositionNode),
    Variable(VariableNode),
}

#[async_trait]
impl NodeTrait for BacktestNode {
    async fn node_id(&self) -> NodeId {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::Indicator(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::IfElse(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::Position(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
            BacktestNode::Variable(node) => node.with_ctx_read(|ctx| ctx.node_id().to_string()).await,
        }
    }
}

impl BacktestNode {
    pub fn klind(&self) -> NodeType {
        match self {
            BacktestNode::Start(_) => NodeType::StartNode,
            BacktestNode::Kline(_) => NodeType::KlineNode,
            BacktestNode::Indicator(_) => NodeType::IndicatorNode,
            BacktestNode::IfElse(_) => NodeType::IfElseNode,
            BacktestNode::FuturesOrder(_) => NodeType::FuturesOrderNode,
            BacktestNode::Position(_) => NodeType::PositionNode,
            BacktestNode::Variable(_) => NodeType::VariableNode,
        }
    }

    pub async fn node_name(&self) -> NodeName {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::Indicator(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::IfElse(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::Position(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
            BacktestNode::Variable(node) => node.with_ctx_read(|ctx| ctx.node_name().to_string()).await,
        }
    }

    pub async fn node_type(&self) -> NodeType {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::Indicator(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::IfElse(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::Position(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
            BacktestNode::Variable(node) => node.with_ctx_read(|ctx| ctx.node_type().clone()).await,
        }
    }

    pub async fn is_in_state(&self, state: NodeRunState) -> bool {
        match self {
            BacktestNode::Start(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::Kline(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::Indicator(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::IfElse(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::FuturesOrder(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::Position(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
            BacktestNode::Variable(node) => {
                node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.is_in_state(&state).await }))
                    .await
            }
        }
    }

    pub async fn run_state(&self) -> NodeRunState {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::Kline(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::Indicator(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::IfElse(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::Position(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
            BacktestNode::Variable(node) => node.with_ctx_read_async(|ctx| Box::pin(async move { ctx.run_state().await })).await,
        }
    }

    pub async fn cancel_token(&self) -> CancellationToken {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::Indicator(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::IfElse(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::Position(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
            BacktestNode::Variable(node) => node.with_ctx_read(|ctx| ctx.cancel_token().clone()).await,
        }
    }

    pub async fn subscribe_strategy_output_handle(&mut self, subscriber_id: String) -> broadcast::Receiver<BacktestNodeEvent> {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::Indicator(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::IfElse(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::Position(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
            BacktestNode::Variable(node) => node.with_ctx_write(|ctx| ctx.subscribe_strategy_bound_handle(subscriber_id)).await,
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
            BacktestNode::Indicator(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            BacktestNode::IfElse(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            BacktestNode::FuturesOrder(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            BacktestNode::Position(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
            BacktestNode::Variable(node) => {
                node.with_ctx_write(|ctx| ctx.subscribe_output_handle(handle_id, subscriber_id))
                    .await
            }
        }
    }

    pub async fn add_input_handle(&self, input_handle: NodeInputHandle<BacktestNodeEvent>) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::Indicator(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::IfElse(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::Position(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
            BacktestNode::Variable(node) => node.with_ctx_write(|ctx| ctx.add_input_handle(input_handle)).await,
        }
    }

    pub async fn set_output_handles(&mut self) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::Indicator(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::IfElse(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::Position(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
            BacktestNode::Variable(node) => node.with_ctx_write(|ctx| ctx.set_output_handles()).await,
        }
    }

    pub async fn set_leaf_node(&mut self, is_leaf_node: bool) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::Indicator(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::IfElse(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::Position(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
            BacktestNode::Variable(node) => node.with_ctx_write(|ctx| ctx.set_leaf_node(is_leaf_node)).await,
        }
    }

    pub async fn add_source_node(&self, source_node_id: String) {
        match self {
            BacktestNode::Start(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::Kline(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::Indicator(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::IfElse(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::Position(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
            BacktestNode::Variable(node) => node.with_ctx_write(|ctx| ctx.add_source_node(source_node_id)).await,
        }
    }

    pub async fn output_handles(&self) -> HashMap<HandleId, NodeOutputHandle<BacktestNodeEvent>> {
        match self {
            BacktestNode::Start(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::Kline(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::Indicator(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::IfElse(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::FuturesOrder(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::Position(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
            BacktestNode::Variable(node) => node.with_ctx_read(|ctx| ctx.output_handles().clone()).await,
        }
    }

    pub async fn init(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.init().await,
            BacktestNode::Kline(node) => node.init().await,
            BacktestNode::Indicator(node) => node.init().await,
            BacktestNode::IfElse(node) => node.init().await,
            BacktestNode::FuturesOrder(node) => node.init().await,
            BacktestNode::Position(node) => node.init().await,
            BacktestNode::Variable(node) => node.init().await,
        }
    }

    pub async fn stop(&self) -> Result<(), BacktestNodeError> {
        match self {
            BacktestNode::Start(node) => node.stop().await,
            BacktestNode::Kline(node) => node.stop().await,
            BacktestNode::Indicator(node) => node.stop().await,
            BacktestNode::IfElse(node) => node.stop().await,
            BacktestNode::FuturesOrder(node) => node.stop().await,
            BacktestNode::Position(node) => node.stop().await,
            BacktestNode::Variable(node) => node.stop().await,
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
