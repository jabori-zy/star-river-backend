use crate::strategy::strategy::Strategy;
use crate::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use crate::node::indicator_node::IndicatorNode;
use event_center::{Event, EventPublisher};





impl Strategy {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i32,
        node_id: String, 
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        indicator: Indicators, 
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let mut node = IndicatorNode::new(
            strategy_id, 
            node_id.clone(), 
            node_name, 
            exchange, 
            symbol, 
            interval, 
            indicator, 
            event_publisher, 
            response_event_receiver,
        );
        // 设置默认输出句柄
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }



    
    
}
