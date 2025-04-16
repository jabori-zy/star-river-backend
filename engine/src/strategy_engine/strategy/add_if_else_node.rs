use crate::strategy_engine::strategy::Strategy;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::if_else_node::IfElseNode;
use event_center::EventPublisher;
use crate::strategy_engine::node::if_else_node::condition::Case;
use crate::strategy_engine::node::NodeTrait;



impl Strategy {
    pub async fn add_if_else_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>, 
        strategy_id: i64,
        node_id: String, 
        node_name: String,
        cases: Vec<Case>,
        event_publisher: EventPublisher,
    ) {
        let mut node = IfElseNode::new(strategy_id, node_id.clone(), node_name.clone(), cases, event_publisher);
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

}
