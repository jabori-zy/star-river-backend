
use super::LiveStrategyFunction;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use crate::strategy_engine::node::order_node::order_node_types::*;
use crate::strategy_engine::node::order_node::OrderNode;
use types::strategy::TradeMode;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;



impl LiveStrategyFunction {
    pub async fn add_order_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i64,
        node_id: String,
        node_name: String,
        trade_mode: TradeMode,
        live_config: Option<OrderNodeLiveConfig>,
        simulate_config: Option<OrderNodeSimulateConfig>,
        backtest_config: Option<OrderNodeBacktestConfig>,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) {
        let mut node = OrderNode::new(
            strategy_id,
            node_id.clone(),
            node_name,
            trade_mode,
            live_config,
            simulate_config,
            backtest_config,
            event_publisher,
            response_event_receiver,
            exchange_engine,
            database,
            heartbeat,
        );
        node.set_output_handle().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }
}