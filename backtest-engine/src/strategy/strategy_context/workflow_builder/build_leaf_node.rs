// third-party
use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use snafu::OptionExt;
use strategy_core::{
    NodeType,
    error::strategy_error::NodeNotFoundByIndexSnafu,
    node::{
        NodeTrait,
        context_trait::{NodeHandleExt, NodeInfoExt},
        node_trait::NodeContextAccessor,
    },
    strategy::{
        context_trait::{StrategyIdentityExt, StrategyWorkflowExt},
        leaf_node_execution_tracker::LeafNodeExecutionInfo,
    },
};

// current crate
use super::BacktestStrategyContext;
use crate::{node::BacktestNode, strategy::strategy_error::BacktestStrategyError};

impl BacktestStrategyContext {
    pub async fn build_leaf_nodes(&mut self) -> Result<(), BacktestStrategyError> {
        let leaf_nodes: Vec<NodeIndex> = self.get_leaf_node_indexs();
        let strategy_name = self.strategy_name().clone();
        let mut leaf_node_ids = Vec::new();
        let mut leaf_node_execution_info = HashMap::new();
        for node_index in leaf_nodes {
            let node = self.node_mut(node_index).context(NodeNotFoundByIndexSnafu {
                strategy_name: strategy_name.clone(),
                node_index: node_index.index(),
            })?;
            let node_id = node.node_id().await;
            node.set_leaf_node(true).await;
            leaf_node_ids.push(node_id.clone());

            // if the node type is if else node, the count == 1
            // Because if-else nodes are executed sequentially, only one case will send a message in a single loop.
            let execute_complete_event_expected_count = if node.node_type().await == NodeType::FuturesOrderNode {
                match node {
                    BacktestNode::FuturesOrder(futures_order_node) => {
                        let (node_name, node_config, input_handles) = futures_order_node
                            .with_ctx_read(|ctx| (ctx.node_name().to_string(), ctx.node_config().clone(), ctx.input_handles().to_vec()))
                            .await;
                        let config_count = node_config.futures_order_configs.len();
                        let input_handles_count = input_handles.len();
                        if input_handles_count == config_count {
                            config_count
                        } else {
                            tracing::warn!("@[{node_name}] have {config_count} order configs, but only {input_handles_count} is connected");
                            input_handles_count
                        }
                    }
                    _ => 0,
                }
            } else if node.node_type().await == NodeType::PositionNode {
                match node {
                    BacktestNode::Position(position_node) => {
                        let (node_name, node_config, input_handles) = position_node
                            .with_ctx_read(|ctx| (ctx.node_name().to_string(), ctx.node_config().clone(), ctx.input_handles().to_vec()))
                            .await;
                        let config_count = node_config.position_operations.len();
                        let input_handles_count = input_handles.len();
                        if input_handles_count == config_count {
                            config_count
                        } else {
                            tracing::warn!(
                                "@[{node_name}] have {config_count} position operations, but only {input_handles_count} is connected"
                            );
                            input_handles_count
                        }
                    }
                    _ => 0,
                }
            } else {
                node.output_handles().await.values().filter(|handle| !handle.is_default()).count()
            };
            let info = LeafNodeExecutionInfo::new(node_id.clone(), execute_complete_event_expected_count as i32);
            leaf_node_execution_info.insert(node_id.clone(), info);
        }
        self.set_leaf_node_ids(leaf_node_ids).await;
        self.set_leaf_node_execution_info(leaf_node_execution_info).await;
        Ok(())
    }
}
