use crate::*;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use types::indicator_config::SMAConfig;
use crate::node::live_data_node::{LiveDataNode, LiveDataNodeState};
use crate::node::indicator_node::{IndicatorNode, IndicatorNodeState};
use crate::node::start_node::StartNode;
use crate::node::if_else_node::{IfElseNode, ComparisonOperator};
use crate::node::buy_node::BuyNode;
use event_center::{Event, EventPublisher};
use std::sync::Arc;
use tokio::sync::RwLock;
use database::entities::strategy_info::Model as StrategyInfo;
use serde_json::Value;
use std::str::FromStr;
use tokio::join;
use crate::node::if_else_node::Case;

#[derive(Debug)]
// 策略图
pub struct Strategy {
    pub id: i32,
    pub name: String,
    pub graph: Graph<Box<dyn NodeTrait>, (),  Directed>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub event_publisher: EventPublisher,
    pub response_event_receiver: broadcast::Receiver<Event>,
}

impl Clone for Strategy {
    fn clone(&self) -> Self {
        Strategy {
            id: self.id,
            name: self.name.clone(),
            graph: self.graph.clone(),
            node_indices: self.node_indices.clone(),
            event_publisher: self.event_publisher.clone(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
        }
    }
}



impl Strategy {
    pub async fn new(
        strategy_info: StrategyInfo, 
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>, 
        response_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        let strategy_id = strategy_info.id;
        let strategy_name = strategy_info.name;
        
        if let Some(nodes_str) = strategy_info.nodes {
            if let Ok(nodes) = serde_json::from_str::<Vec<Value>>(&nodes_str.to_string()) {
                for node_config in nodes {
                    // tracing::debug!("node_config: {:?}", node_config);
                    Self::add_node(
                        &mut graph, 
                        &mut node_indices, 
                        &node_config, 
                        event_publisher.clone(), 
                        market_event_receiver.resubscribe(), 
                        response_event_receiver.resubscribe()
                    ).await;

                }
            }
        }
        // 添加边
        if let Some(edges_str) = strategy_info.edges {
            if let Ok(edges) = serde_json::from_str::<Vec<Value>>(&edges_str.to_string()) {
                // tracing::debug!("edges: {:?}", edges);
                for edge_config in edges {
                    let source_handle = edge_config["sourceHandle"].as_str().unwrap();
                    let from_node_id = edge_config["source"].as_str().unwrap();
                    let to_node_id = edge_config["target"].as_str().unwrap();

                    Self::add_edge(&mut graph, &mut node_indices, from_node_id, source_handle, to_node_id).await;
                }
            }
        }

        Self {
            id: strategy_id,
            name: strategy_name,
            graph,
            node_indices,
            event_publisher,
            response_event_receiver,
        }
    }

    async fn add_node(
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
                Self::add_start_node(
                    graph, 
                    node_indices,
                    node_id.to_string(), 
                    node_name.to_string()
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
                    response_event_receiver
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
                    response_event_receiver
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
                    cases
                ).await;
            }
            // 买入节点
            NodeType::BuyNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                Self::add_buy_node(
                    graph,
                    node_indices,
                    node_id.to_string(),
                    node_name.to_string()
                ).await;
                
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
            }
            
        }

    }

    async fn add_start_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String, 
        node_name: String
    ) {
        let node = StartNode::new(node_id.clone(), node_name).init_node().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    pub async fn add_live_data_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        strategy_id: i32,
        node_id: String,
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        event_publisher: EventPublisher,
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node = LiveDataNode::new(strategy_id, node_id.clone(), node_name, exchange, symbol, interval, event_publisher, market_event_receiver, response_event_receiver).init_node().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

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
        response_event_receiver: broadcast::Receiver<Event>
    ) {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node = IndicatorNode::new(strategy_id, node_id.clone(), node_name, exchange, symbol, interval, indicator, event_publisher, response_event_receiver).init_node().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    pub async fn add_if_else_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>, 
        node_id: String, 
        node_name: String,
        cases: Vec<Case>
    ) {
        let node = Box::new(IfElseNode::new(node_id.clone(), node_name.clone(), cases).init_node().await);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    pub async fn add_buy_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String,
        node_name: String
    ) {
        let node = Box::new(BuyNode::new(node_id.clone(), node_name.clone()).init_node().await);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    // 添加边
    pub async fn add_edge(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        from_node_id: &str,
        from_handle_id: &str,
        to_node_id: &str
    ) {
        if let (Some(&source), Some(&target)) = (
            node_indices.get(from_node_id),
            node_indices.get(to_node_id)
        ){

            // 先获取源节点的发送者
            let sender = graph.node_weight(source).unwrap().get_node_sender(from_handle_id.to_string()).await;
            tracing::debug!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let receiver_count = sender.receiver_count();
                tracing::debug!("{:?} 添加了一个接收者, 接收者数量 = {}", target_node.get_node_name().await, receiver_count);
                target_node.add_message_receiver(receiver);
                target_node.add_from_node_id(from_node_id.to_string());
            }
            tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
            graph.add_edge(source, target, ());
        }
        

    }

    pub fn topological_sort(&self) -> Vec<&Box<dyn NodeTrait>> {
        petgraph::algo::toposort(&self.graph, None)
        .unwrap_or_default()
        .into_iter()
        .map(|index| &self.graph[index])
        .collect()
    }

    pub async fn run(&mut self) {
        let nodes = self.topological_sort();  // 假设这个方法返回按拓扑排序的节点

        for node in nodes {
            let mut node = node.clone();
            // 使用 tokio::join! 确保节点按顺序执行
            let handle = tokio::spawn(async move {
                node.run().await.unwrap();
            });

            // 等待当前节点完成后再继续
            join!(handle).0.unwrap();
        }
    }
}

    




