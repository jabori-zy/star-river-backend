use crate::strategy::strategy::Strategy;
use crate::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use event_center::{Event, EventPublisher};
use serde_json::Value;
use std::str::FromStr;
use crate::node::if_else_node::Case;
use crate::NodeType;

impl Strategy {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: &Value,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let event_publisher = event_publisher.clone();
                Self::add_start_node(
                    graph, 
                    node_indices,
                    node_id.to_string(), 
                    node_name.to_string(),
                    event_publisher,
                    
                ).await;
            }
            // 指标节点
            NodeType::IndicatorNode => {
                let node_data = node_config["data"].clone();
                let strategy_id = node_data["strategyId"].as_i64().unwrap() as i32;
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();

                let indicator_name = node_data["indicatorName"].as_str().unwrap_or_default(); // 指标名称
                let mut indicator = Indicators::from_str(indicator_name).unwrap(); // 转换成指标
                let indicator_config = node_data["indicatorConfig"].clone(); // 指标配置
                indicator.update_config(&indicator_config); // 更新指标配置

                let exchange = Exchange::Binance; // 交易所
                let symbol = "BTCUSDT".to_string();
                let interval = KlineInterval::Minutes1;
                
                let response_event_receiver = response_event_receiver.resubscribe();
                Self::add_indicator_node(
                    graph, 
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    exchange, 
                    symbol, 
                    interval, 
                    indicator,
                    event_publisher.clone(),
                    response_event_receiver,
                ).await;
                
            }
            // 实时数据节点
            NodeType::LiveDataNode => {
                let node_data = node_config["data"].clone();
                let strategy_id = node_data["strategyId"].as_i64().unwrap() as i32;
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let exchange = node_data["exchange"].as_str().unwrap();
                let symbol = node_data["symbol"].as_str().unwrap();
                let interval = node_data["interval"].as_str().unwrap();
                let event_publisher = event_publisher.clone();
                let market_event_receiver = market_event_receiver.resubscribe();
                let response_event_receiver = response_event_receiver.resubscribe();

                Self::add_live_data_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    Exchange::from_str(exchange).unwrap(), 
                    symbol.to_string(), 
                    KlineInterval::from_str(interval).unwrap(), 
                    event_publisher,
                    market_event_receiver,
                    response_event_receiver,
                ).await;
                
            }
            // 条件分支节点
            NodeType::IfElseNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let cases: Vec<Case> = serde_json::from_value(node_data["cases"].clone())
                    .unwrap_or_else(|e| panic!("Failed to parse cases: {}", e));
                Self::add_if_else_node(
                    graph,
                    node_indices,
                    node_id.to_string(),
                    node_name.to_string(),
                    cases,
                    event_publisher,
                ).await;
            }
            // 买入节点
            NodeType::BuyNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let event_publisher = event_publisher.clone();
                Self::add_buy_node(
                    graph,
                    node_indices,
                    node_id.to_string(),
                    node_name.to_string(),
                    event_publisher,
                ).await;
                
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
            }
            
        }

    }



    
    
}
