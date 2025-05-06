use super::LiveStrategyFunction;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::live_strategy_node::if_else_node::IfElseNode;
use event_center::EventPublisher;
use crate::strategy_engine::node::live_strategy_node::if_else_node::if_else_node_type::*;
use crate::strategy_engine::node::NodeTrait;
use crate::strategy_engine::node::live_strategy_node::if_else_node::condition::Case;

impl LiveStrategyFunction {
    pub async fn add_if_else_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>, 
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
    ) -> Result<(), String> {

        let node_data = node_config["data"].clone();
        let node_id = node_config["id"].as_str().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap_or_default();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let live_config_json = node_data["liveConfig"].clone();
        if live_config_json.is_null() {
            return Err("liveConfig is null".to_string());
        }
        let cases: Vec<Case> = serde_json::from_value(live_config_json["cases"].clone()).unwrap();
        let if_else_node_live_config = IfElseNodeLiveConfig {
            cases: cases.clone(),
        };
        tracing::debug!("条件分支节点数据: {:?}", if_else_node_live_config);

        let mut node = IfElseNode::new(
            strategy_id as i32, 
            node_id.to_string(),
            node_name.to_string(), 
            if_else_node_live_config, 
            event_publisher,
        );
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }

}
