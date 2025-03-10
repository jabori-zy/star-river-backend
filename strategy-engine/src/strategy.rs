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
use crate::node::condition_node::{ConditionNode, Condition, ConditionType, Operator};
use event_center::{Event, EventPublisher};
use std::sync::Arc;
use tokio::sync::RwLock;
use database::entities::strategy_info::Model as StrategyInfo;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone)]
// 策略图
pub struct Strategy {
    pub id: i32,
    pub name: String,
    pub graph: Graph<Box<dyn NodeTrait>, (),  Directed>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl Strategy {
    pub async fn new(strategy_info: StrategyInfo, event_publisher: EventPublisher, market_event_receiver: broadcast::Receiver<Event>) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        let strategy_id = strategy_info.id;
        let strategy_name = strategy_info.name;
        
        if let Some(nodes_str) = strategy_info.nodes {
            if let Ok(nodes) = serde_json::from_str::<Vec<Value>>(&nodes_str.to_string()) {
                for node_config in nodes {
                    tracing::debug!("node_config: {:?}", node_config);
                    Self::add_node(&mut graph, &mut node_indices, &node_config, event_publisher.clone(), market_event_receiver.resubscribe());

                }
            }
        }
        // 添加边
        if let Some(edges_str) = strategy_info.edges {
            if let Ok(edges) = serde_json::from_str::<Vec<Value>>(&edges_str.to_string()) {
                tracing::debug!("edges: {:?}", edges);
                for edge_config in edges {
                    let from = edge_config["source"].as_str().unwrap();
                    let to = edge_config["target"].as_str().unwrap();

                    Self::add_edge(&mut graph, &mut node_indices, from, to).await;
                }
            }
        }

        Self {
            id: strategy_id,
            name: strategy_name,
            graph,
            node_indices,
        }
    }

    fn add_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: &Value,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>) {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = "起始节点".to_string();
                Self::add_start_node(
                    graph, 
                    node_indices,
                    node_id.to_string(), 
                    node_name.to_string()
                );
            }
            // 指标节点
            NodeType::IndicatorNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();

                let indicator_name = node_data["indicatorName"].as_str().unwrap_or_default();

                let mut indicator = Indicators::from_str(indicator_name).unwrap();
                let indicator_config = node_data["indicatorConfig"].clone();
                indicator.update_config(&indicator_config);
                let node_name = "指标节点".to_string();
                let exchange = Exchange::Binance;
                let symbol = "BTCUSDT".to_string();
                let interval = KlineInterval::Minutes1;
                
                let response_event_receiver = market_event_receiver.resubscribe();
                Self::add_indicator_node(
                    graph, 
                    node_indices,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    exchange, 
                    symbol, 
                    interval, 
                    indicator,
                    event_publisher.clone(),
                    response_event_receiver
                );
                
            }
            // 实时数据节点
            NodeType::LiveDataNode => {
                let node_id = node_config["id"].as_str().unwrap();
                // let node_name = node_config["name"].as_str().unwrap();
                let node_name = "实时数据节点".to_string();
                let node_data = node_config["data"].clone();
                let exchange = node_data["exchange"].as_str().unwrap();
                let symbol = node_data["symbol"].as_str().unwrap();
                let interval = node_data["interval"].as_str().unwrap();
                let market_event_receiver = market_event_receiver.resubscribe();
                Self::add_live_data_node(
                    graph,
                    node_indices,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    Exchange::from_str(exchange).unwrap(), 
                    symbol.to_string(), 
                    KlineInterval::from_str(interval).unwrap(), 
                    market_event_receiver
                );
                
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
            }
            
        }

    }

    fn add_start_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String, 
        node_name: String
    ) {
        let node = StartNode::new(node_id.clone(), node_name);
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }



    pub fn add_live_data_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String,
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        market_event_receiver: broadcast::Receiver<Event>
    ) {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node = LiveDataNode {
            state: Arc::new(RwLock::new(LiveDataNodeState {
                node_id: node_id.clone(),    
                node_name,
                exchange,
                symbol,
                interval,
                node_sender: NodeSender::new(node_id.clone(), tx),
            })),
            node_receivers: Vec::new(),
            node_type: NodeType::DataSourceNode,
            market_event_receiver,
        };
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    pub fn add_indicator_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
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

        let node = IndicatorNode {
            node_type: NodeType::IndicatorNode,
            node_receivers: Vec::new(), 
            event_publisher,
            response_event_receiver,
            state: Arc::new(RwLock::new(IndicatorNodeState {
                node_id: node_id.clone(),
                node_name,
                exchange,
                symbol,
                interval,
                indicator,
                current_batch_id: None,
                request_id: None,
                node_sender: NodeSender::new(node_id.clone(), tx),
            })),
        };
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
    }

    // pub fn add_condition_node(&mut self, condition_node: ConditionNode) -> i32 {
    //     let node_id = condition_node.node_id;
    //     let node = Box::new(condition_node);
    //     let node_index = self.graph.add_node(node);
    //     self.node_indices.insert(node_id, node_index);
    //     node_id
    // }

    pub async fn add_edge(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        from_id: &str, 
        to_id: &str
    ) {
        if let (Some(&source), Some(&target)) = (
            node_indices.get(from_id),
            node_indices.get(to_id)
        ){

            // 先获取源节点的发送者
            let sender = graph.node_weight(source).unwrap().get_sender().await;
            println!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                // let receiver_count = sender.receiver_count();
                // println!("{} 添加了一个接收者, 接收者数量 = {}", target_node.name, receiver_count);
                target_node.push_receiver(receiver);
            }
            println!("添加边: {:?} -> {:?}", source, target);
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
        let mut handles = Vec::new();

        let nodes = self.topological_sort();

        for node in nodes {
            let mut node = node.clone();
            let handle = tokio::spawn(async move {
                node.run().await.unwrap();
            });
            handles.push(handle);
        }
    }
}

    




