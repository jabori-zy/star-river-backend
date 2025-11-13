use std::collections::HashMap;

use star_river_core::custom_type::NodeId;

#[derive(Debug)]
pub struct LeafNodeExecutionInfo {
    pub node_id: NodeId,
    pub expected_count: i32,
    pub actual_count: i32,
}

impl LeafNodeExecutionInfo {
    pub fn new(node_id: NodeId, expected_count: i32) -> Self {
        Self {
            node_id,
            expected_count,
            actual_count: 0,
        }
    }

    pub fn increment_actual_count(&mut self) {
        self.actual_count += 1;
    }

    pub fn is_completed(&self) -> bool {
        self.actual_count == self.expected_count
    }

    pub fn reset(&mut self) {
        self.actual_count = 0;
    }
}

#[derive(Debug)]
pub struct LeafNodeExecutionTracker {
    pub leaf_node_ids: Vec<NodeId>,
    // the number of the expected "execute over" event of the leaf node should be received
    pub leaf_node_execution_info: HashMap<NodeId, LeafNodeExecutionInfo>,
}

impl LeafNodeExecutionTracker {
    pub fn new() -> Self {
        Self {
            leaf_node_ids: Vec::new(),
            leaf_node_execution_info: HashMap::new(),
        }
    }

    pub fn execute_completed(&mut self, node_id: NodeId) {
        // if the node is a leaf node
        if self.leaf_node_ids.contains(&node_id) {
            self.leaf_node_execution_info
                .get_mut(&node_id)
                .map(|info| info.increment_actual_count());
        }
    }

    pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_ids = leaf_node_ids;
    }

    pub fn set_leaf_node_execution_info(&mut self, execution_info: HashMap<NodeId, LeafNodeExecutionInfo>) {
        self.leaf_node_execution_info = execution_info;
    }

    pub fn is_all_completed(&self) -> bool {
        self.leaf_node_execution_info.values().all(|info| info.is_completed())
    }

    pub fn reset(&mut self) {
        self.leaf_node_execution_info.iter_mut().for_each(|(_, info)| info.reset());
    }
}
