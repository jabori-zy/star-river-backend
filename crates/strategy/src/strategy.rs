use crate::*;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use crate::data_source_node::DataSourceNode;
use crate::indicator_node::IndicatorNode;
use crate::condition_node::{ConditionNode, Condition, ConditionType, Operator};
use event_center::{Event, EventPublisher};

// 策略图
pub struct Strategy {
    pub id: Uuid,
    pub name: String,
    pub graph: Graph<Box<dyn NodeTrait>, (),  Directed>,
    pub node_indices: HashMap<Uuid, NodeIndex>,
}

impl Strategy {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_data_source_node(&mut self, name: String, exchange: Exchange, symbol: String, interval: KlineInterval, market_event_receiver: broadcast::Receiver<Event>) -> Uuid {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        let node = DataSourceNode {
            id: node_id,    
            name,
            node_type: NodeType::DataSource,
            exchange,
            symbol,
            interval,
            sender: NodeSender::new(node_id.to_string(), tx),
            receivers: Vec::new(),
            market_event_receiver,
        };
        let node = Box::new(node);
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(node_id, node_index);
        node_id
    }

    pub fn add_indicator_node(&mut self, name: String, exchange: Exchange, symbol: String, interval: KlineInterval, indicator: Indicators, event_publisher: EventPublisher, response_event_receiver: broadcast::Receiver<Event>) -> Uuid {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        let node = IndicatorNode {
            id: node_id,
            name,
            node_type: NodeType::Indicator,
            exchange,
            symbol,
            interval,
            indicator,
            node_sender: NodeSender::new(node_id.to_string(), tx),
            node_receivers: Vec::new(), 
            event_publisher,
            response_event_receiver,
            current_batch_id: None,
            request_id: None,
        };
        let node = Box::new(node);
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(node_id, node_index);
        node_id
    }

    pub fn add_condition_node(&mut self, condition_node: ConditionNode) -> Uuid {
        let node_id = condition_node.id;
        let node = Box::new(condition_node);
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(node_id, node_index);
        node_id
    }

    pub fn add_edge(&mut self, from: &Uuid, to: &Uuid) {
        if let (Some(&source), Some(&target)) = (
            self.node_indices.get(from),
            self.node_indices.get(to)
        ){
            // 先获取源节点的发送者
            let sender = self.graph.node_weight(source).unwrap().get_sender();
            println!("sender: {:?}", sender);

            if let Some(target_node) = self.graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let receiver_count = sender.receiver_count();
                println!("{} 添加了一个接收者, 接收者数量 = {}", target_node.id(), receiver_count);
                target_node.push_receiver(receiver);
            }
            println!("添加边: {:?} -> {:?}", source, target);
            self.graph.add_edge(source, target, ());
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
        println!("Graph: {:?}", self.graph);
        let nodes = self.topological_sort();
        println!("nodes: {:?}", nodes);
        for node in nodes {
            let mut node = node.clone();
            let handle = tokio::spawn(async move {
                node.run().await.unwrap();
            });
            handles.push(handle);
        }
    }
}

    




