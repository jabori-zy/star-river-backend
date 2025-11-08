// third-party
use petgraph::{Direction, graph::NodeIndex};

// current crate
use super::BacktestStrategyContext;
use strategy_core::strategy::context_trait::StrategyWorkflowExt;
use strategy_core::node::NodeTrait;


impl BacktestStrategyContext {
    pub async fn build_leaf_nodes(
        &mut self,
    ) {
        let leaf_nodes: Vec<NodeIndex> = self.get_leaf_node_indexs();
        let mut leaf_node_ids = Vec::new();
        for node_index in leaf_nodes {
            if let Some(node) = self.node_mut(node_index) {
                let node_id = node.node_id().await;
                node.set_leaf_node(true).await;
                leaf_node_ids.push(node_id);
            }
        }
        self.set_leaf_node_ids(leaf_node_ids);
    }
}