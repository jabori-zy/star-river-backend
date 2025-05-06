use super::LiveStrategyFunction;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::live_strategy_node::start_node::StartNode;
use event_center::EventPublisher;
use crate::strategy_engine::node::NodeTrait;
use types::strategy::{LiveConfig, BacktestConfig, SimulatedConfig, TradeMode};


impl LiveStrategyFunction {
    pub async fn add_start_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
    ) -> Result<(), String> {

        let node_data = node_config["data"].clone();
        let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
        let node_id = node_config["id"].as_str().unwrap(); // 节点id
        let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
        // tracing::debug!("开始节点数据: {:?}, {:?}, {:?}", node_data["liveConfig"], node_data["backtestConfig"], node_data["simulatedConfig"]);
        // 解析策略配置
        let live_config_json = node_data["liveConfig"].clone();
        if live_config_json.is_null() {
            return Err("liveConfig is null".to_string());
        }
        let live_config = serde_json::from_value::<LiveConfig>(live_config_json).unwrap();
        let mut node = StartNode::new(
            strategy_id as i32, 
            node_id.to_string(), 
            node_name.to_string(), 
            live_config, 
            event_publisher
        );
        // 设置默认输出句柄
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }
}
