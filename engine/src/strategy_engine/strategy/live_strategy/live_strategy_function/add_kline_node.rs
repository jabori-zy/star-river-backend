use super::LiveStrategyFunction;
use tokio::sync::Mutex;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use crate::strategy_engine::node::live_strategy_node::kline_node::KlineNode;
use event_center::{Event, EventPublisher};
use crate::strategy_engine::node::LiveNodeTrait;
use crate::strategy_engine::node::live_strategy_node::kline_node::kline_node_context::KlineNodeLiveConfig;
use std::sync::Arc;
use heartbeat::Heartbeat;
use types::cache::Key;
use types::cache::key::KlineKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;

impl LiveStrategyFunction {
    pub async fn add_kline_node(
        graph: &mut Graph<Box<dyn LiveNodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<Key>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: NodeCommandSender,
        ) -> Result<(), String> {
            let node_data = node_config["data"].clone();
            let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
            let node_id = node_config["id"].as_str().unwrap(); // 节点id
            let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
            
            // k线频率设置
            let frequency = 2000;
            tracing::debug!("实时数据节点数据: {:?}", node_data);
            // 解析策略配置
            let live_config_json = node_data["liveConfig"].clone();
            if live_config_json.is_null() {
                return Err("liveConfig is null".to_string());
            }
            let live_config = serde_json::from_value::<KlineNodeLiveConfig>(live_config_json).unwrap();
            let cache_key = KlineKey::new(live_config.selected_live_account.exchange.clone(), live_config.symbol.clone(), live_config.interval.clone());
            cache_keys.push(cache_key.into());
            
            let mut node = KlineNode::new(
                strategy_id as i32,
                node_id.to_string(), 
                node_name.to_string(), 
                live_config,
                event_publisher,
                command_publisher,
                command_receiver,
                market_event_receiver,
                response_event_receiver,
                heartbeat,
                strategy_command_sender,
            );
            // 设置默认输出句柄
            node.set_output_handle().await;

            let node = Box::new(node);
            let node_index = graph.add_node(node);
            node_indices.insert(node_id.to_string(), node_index);
            Ok(())
        }
}