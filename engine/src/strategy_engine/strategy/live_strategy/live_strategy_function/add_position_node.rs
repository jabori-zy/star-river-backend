
use super::LiveStrategyFunction;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use crate::strategy_engine::node::live_strategy_node::position_node::PositionNode;
use crate::strategy_engine::node::live_strategy_node::position_node::position_node_types::*;



impl LiveStrategyFunction {
    pub async fn add_position_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone();
        let node_id = node_config["id"].as_str().unwrap().to_string();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap().to_string();
        let live_config_json = node_data["liveConfig"].clone();
        if live_config_json.is_null() {
            return Err("liveConfig is null".to_string());
        };
        let live_config = serde_json::from_value::<PositionNodeLiveConfig>(live_config_json).unwrap();
        let mut node = PositionNode::new(
            strategy_id as i32,
            node_id.clone(),
            node_name,
            live_config,
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
        Ok(())
    }
}