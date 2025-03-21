use crate::strategy::strategy::Strategy;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::node::if_else_node::IfElseNode;
use event_center::EventPublisher;
use crate::node::if_else_node::Case;
use crate::node::NodeTrait;


impl Strategy {
    pub async fn add_if_else_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>, 
        node_id: String, 
        node_name: String,
        cases: Vec<Case>,
        event_publisher: EventPublisher
    ) {
        let node = Box::new(IfElseNode::new(node_id.clone(), node_name.clone(), cases, event_publisher).init_node().await);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

}
