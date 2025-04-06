
use crate::strategy::strategy::Strategy;
use crate::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::Exchange;
use types::order::OrderRequest;
use crate::node::order_node::OrderNode;
use event_center::{Event, EventPublisher};




impl Strategy {
    pub async fn add_order_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i32,
        node_id: String,
        node_name: String,
        exchange: Exchange,
        symbol: String,
        order_request: OrderRequest,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let mut node = OrderNode::new(
            strategy_id,
            node_id.clone(),
            node_name,
            exchange,
            symbol,
            order_request,
            event_publisher,
            response_event_receiver,
        );
        node.set_output_handle().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }
}