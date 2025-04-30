use crate::strategy_engine::strategy::Strategy;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use crate::strategy_engine::node::indicator_node::IndicatorNode;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use crate::strategy_engine::node::indicator_node::indicator_node_type::{IndicatorNodeLiveConfig, IndicatorNodeBacktestConfig, IndicatorNodeSimulateConfig};



impl Strategy {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i64,
        node_id: String, 
        node_name: String, 
        live_config: Option<IndicatorNodeLiveConfig>,
        backtest_config: Option<IndicatorNodeBacktestConfig>,
        simulated_config: Option<IndicatorNodeSimulateConfig>,
        trade_mode: TradeMode,
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let mut node = IndicatorNode::new(
            strategy_id, 
            node_id.clone(), 
            node_name, 
            trade_mode,
            live_config,
            backtest_config,
            simulated_config,
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
