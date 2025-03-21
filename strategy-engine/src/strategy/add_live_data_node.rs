use crate::strategy::strategy::Strategy;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::{broadcast, watch};
use types::market::{Exchange, KlineInterval};
use crate::node::live_data_node::LiveDataNode;
use event_center::{Event, EventPublisher};
use crate::strategy::strategy::StrategyState;
use crate::node::NodeTrait;
use tokio_util::sync::CancellationToken;

impl Strategy {
    pub async fn add_live_data_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i32,
        node_id: String,
        node_name: String, 
            exchange: Exchange, 
            symbol: String, 
            interval: KlineInterval, 
            event_publisher: EventPublisher,
            market_event_receiver: broadcast::Receiver<Event>,
            response_event_receiver: broadcast::Receiver<Event>,
            strategy_state_rx: watch::Receiver<StrategyState>,
        ) {
            let node = LiveDataNode::new(
                strategy_id,
                node_id.clone(), 
                node_name, 
                exchange, 
                symbol, 
                interval, 
                event_publisher, 
                market_event_receiver, 
                response_event_receiver,
                strategy_state_rx,
            ).init_node().await;

            let node = Box::new(node);
            let node_index = graph.add_node(node);
            node_indices.insert(node_id, node_index);
        }
}