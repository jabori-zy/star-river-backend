use super::LiveStrategyFunction;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::IndicatorConfig;
use crate::strategy_engine::node::live_strategy_node::indicator_node::IndicatorNode;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use crate::strategy_engine::node::live_strategy_node::indicator_node::indicator_node_type::{IndicatorNodeLiveConfig, IndicatorNodeBacktestConfig, IndicatorNodeSimulateConfig};
use std::str::FromStr;
use types::cache::CacheKey;
use types::cache::cache_key::IndicatorCacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;

impl LiveStrategyFunction {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<CacheKey>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        strategy_command_sender: NodeCommandSender,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let node_id = node_config["id"].as_str().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap_or_default();

        let indicator_type = node_data["indicatorType"].as_str().unwrap_or_default(); // 指标类型
        let live_config_json = node_data["liveConfig"].clone();
        if live_config_json.is_null() {
            return Err("liveConfig is null".to_string());
        };
                
        let indicator_config = live_config_json["indicatorConfig"].clone();
        let indicator_config = IndicatorConfig::new(indicator_type, &indicator_config);
        let symbol = live_config_json["symbol"].as_str().unwrap_or_default().to_string();
        let interval = KlineInterval::from_str(live_config_json["interval"].as_str().unwrap_or_default()).unwrap();
        let exchange = Exchange::from_str(live_config_json["exchange"].as_str().unwrap_or_default()).unwrap();
        let indicator_node_live_config = IndicatorNodeLiveConfig {
            indicator_config: indicator_config.clone(),
            symbol: symbol.clone(),
            interval: interval.clone(),
            exchange: exchange.clone(),
        };
        let cache_key = IndicatorCacheKey::new(exchange, symbol, interval, indicator_config);
        cache_keys.push(cache_key.into());

        let mut node = IndicatorNode::new(
            strategy_id as i32,
            node_id.to_string(), 
            node_name.to_string(), 
            indicator_node_live_config,
            event_publisher,
            command_publisher,
            command_receiver,
            response_event_receiver,
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
