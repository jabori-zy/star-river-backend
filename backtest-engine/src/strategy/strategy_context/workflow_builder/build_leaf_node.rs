// third-party
use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use snafu::OptionExt;
use strategy_core::{
    NodeType,
    error::strategy_error::{NodeNotFoundByIndexSnafu, StrategyError},
    node::NodeTrait,
    strategy::{
        context_trait::{StrategyIdentityExt, StrategyWorkflowExt},
        leaf_node_execution_tracker::LeafNodeExecutionInfo,
    },
};

// current crate
use super::BacktestStrategyContext;

impl BacktestStrategyContext {
    pub async fn build_leaf_nodes(&mut self) -> Result<(), StrategyError> {
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
            let execute_complete_event_expected_count = if node.node_type().await == NodeType::IfElseNode {
                1
            } else {
                node.output_handles()
                    .await
                    .values()
                    .filter(
                        |handle: &&strategy_core::node::node_handles::NodeOutputHandle<crate::node::node_event::BacktestNodeEvent>| {
                            !handle.is_default()
                        },
                    )
                    .count()
            };
            let info = LeafNodeExecutionInfo::new(node_id.clone(), execute_complete_event_expected_count as i32);
            leaf_node_execution_info.insert(node_id.clone(), info);
        }
        self.set_leaf_node_ids(leaf_node_ids).await;
        self.set_leaf_node_execution_info(leaf_node_execution_info).await;
        Ok(())
    }
}
