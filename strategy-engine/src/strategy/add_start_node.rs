use crate::strategy::strategy::Strategy;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::node::start_node::StartNode;
use event_center::EventPublisher;
use crate::node::NodeTrait;


impl Strategy {
    pub async fn add_start_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String, 
        node_name: String,
        event_publisher: EventPublisher
    ) {
        let node = StartNode::new(node_id.clone(), node_name, event_publisher).init_node().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }
}
