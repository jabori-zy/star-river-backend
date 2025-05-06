use super::LiveStrategyFunction;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use crate::strategy_engine::node::live_strategy_node::indicator_node::IndicatorNode;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use crate::strategy_engine::node::live_strategy_node::indicator_node::indicator_node_type::{IndicatorNodeLiveConfig, IndicatorNodeBacktestConfig, IndicatorNodeSimulateConfig};
use std::str::FromStr;


impl LiveStrategyFunction {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let node_id = node_config["id"].as_str().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap_or_default();

        let indicator_type = node_data["indicatorType"].as_str().unwrap_or_default(); // 指标类型
        let mut indicator = Indicators::from_str(indicator_type).unwrap(); // 转换成指标
        let live_config_json = node_data["liveConfig"].clone();
        if live_config_json.is_null() {
            return Err("liveConfig is null".to_string());
        };
                
        let indicator_config = live_config_json["indicatorConfig"].clone();
        indicator.update_config(&indicator_config); // 更新指标配置
        let symbol = live_config_json["symbol"].as_str().unwrap_or_default().to_string();
        let interval = KlineInterval::from_str(live_config_json["interval"].as_str().unwrap_or_default()).unwrap();
        let exchange = Exchange::from_str(live_config_json["exchange"].as_str().unwrap_or_default()).unwrap();
        let indicator_node_live_config = IndicatorNodeLiveConfig {
            indicator: indicator.clone(),
            symbol,
            interval,
            exchange,
        };

        let mut node = IndicatorNode::new(
            strategy_id as i32,
            node_id.to_string(), 
            node_name.to_string(), 
            indicator_node_live_config,
            event_publisher, 
            response_event_receiver,
        );
        // 设置默认输出句柄
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }



    
    
}
