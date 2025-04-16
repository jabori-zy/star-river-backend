
use crate::strategy_engine::strategy::Strategy;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::Exchange;
use event_center::{Event, EventPublisher};
use crate::strategy_engine::node::order_node::order_node_types::OrderConfig;
use crate::strategy_engine::node::order_node::OrderNode;




impl Strategy {
    pub async fn add_order_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i64,
        node_id: String,
        node_name: String,
        exchange: Exchange,
        symbol: String,
        order_config: OrderConfig,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let mut node = OrderNode::new(
            strategy_id,
            node_id.clone(),
            node_name,
            exchange,
            symbol,
            order_config,
            event_publisher,
            response_event_receiver,
        );
        node.set_output_handle().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }
}