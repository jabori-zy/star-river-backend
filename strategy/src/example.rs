use std::collections::HashMap;

use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use tokio::sync::broadcast;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use futures::StreamExt;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone, Debug)]
pub enum NodeType {
    DataSource,
    Indicator(String),
    Condition,
}

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub sender:broadcast::Sender<NodeData>,
    pub receivers: Vec<broadcast::Receiver<NodeData>>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            id: self.id.clone(),
            node_type: self.node_type.clone(),
            sender: self.sender.clone(),
            receivers: self.receivers.iter().map(|rx| rx.resubscribe()).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Price(f64),
    Indicator(f64),
    Signal(bool),
}

#[derive(Clone, Debug)]
pub struct DataFetchConfig {
    pub symbol: String,
    pub interval: String,
    pub data_type: String,
}

#[derive(Clone, Debug)]
pub struct IndicatorConfig {
    pub indicator_type: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ConditionConfig {
    pub condition_type: String,
    pub parameters: HashMap<String, String>,
}


pub struct Strategy {
    pub id: String,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}




pub struct StrategyEngine {
    strategy: Strategy,
}

impl StrategyEngine {
    pub fn new(strategy: Strategy) -> Self {
        Self { strategy }
    }

    // fn build_execution_graph(&self) -> Result<ExecutionGraph, String> {
    //     let mut graph = Graph::<Node, (), Directed>::new();
    //     let mut node_indices = HashMap::new();

    //     for node in &self.strategy.nodes {
    //         let node_index = graph.add_node(node.clone());
    //         node_indices.insert(node.id.clone(), node_index);
    //     }

    //     for edge in &self.strategy.edges {
    //         let source_index = node_indices.get(&edge.source)
    //             .ok_or_else(|| format!("Source node not found: {}", edge.source))?;
    //         let target_index = node_indices.get(&edge.target)
    //             .ok_or_else(|| format!("Target node not found: {}", edge.target))?;
            
    //         graph.add_edge(*source_index, *target_index, ());
    //     }

    //     Ok(ExecutionGraph {
    //         graph,
    //         node_indices,
    //     })
            
    // }

    // pub async fn execute(&self) -> Result<(), String> {
    //     let graph = self.build_execution_graph()?;
    //     let sorted_nodes = graph.topological_sort();
    //     println!("Sorted nodes: {:?}", sorted_nodes);
    //     Ok(())

    // }
    
    
}

#[derive(Debug)]
pub struct ExecutionGraph {
    graph: Graph<Node, (), Directed>,
    node_indices: HashMap<String, NodeIndex>,
}

impl ExecutionGraph {

    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn get_senders(&self, node_index: NodeIndex) -> broadcast::Sender<NodeData> {
        // 获取指定节点的所有发送者
        if let Some(node) = self.graph.node_weight(node_index) {
            node.sender.clone()
        } else {
            panic!("节点不存在");
        }
    }

    pub fn get_receivers(&self, node_index: NodeIndex) -> Vec<broadcast::Receiver<NodeData>> {
        if let Some(node) = self.graph.node_weight(node_index) {
            node.receivers.iter()
                .map(|rx| rx.resubscribe())
                .collect()
        } else {
            vec![]
        }
    }
    

    pub fn add_node(&mut self, id: String, node_type: NodeType) -> NodeIndex {
        let (tx, rx) = broadcast::channel::<NodeData>(100);
        let node = Node {
            id: id.clone(),
            node_type,
            sender: tx,
            receivers: Vec::new(),
        };
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(id, node_index);
        node_index
    }

    pub fn add_edge(&mut self, from: &str, to: &str){
        if let (Some(&source), Some(&target)) = (
            self.node_indices.get(from),
            self.node_indices.get(to)
        ){
            // let (tx, rx) = broadcast::channel::<NodeData>(100);
            // // 添加发送者到源节点
            // if let Some(source_node) = self.graph.node_weight_mut(source) {
            //     println!("源节点: {:?} 添加了一个发送者", source_node.id);
            //     source_node.senders.push(tx);
            // }

            // 先获取源节点的发送者
            let sender = self.graph.node_weight(source)
                .map(|node| node.sender.clone())
                .unwrap();
            
            
            if let Some(target_node) = self.graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let receiver_count = sender.receiver_count();
                println!("{} 添加了一个接收者, 接收者数量 = {}", target_node.id, receiver_count);
                target_node.receivers.push(receiver);
            }
            self.graph.add_edge(source, target, ());
        }
    }

    pub async fn run(&mut self) {
        let mut handles = Vec::new();
        println!("Graph: {:?}", self.graph);

        // 获取拓扑排序
        let nodes: Vec<_> = petgraph::algo::toposort(&self.graph, None)
            .unwrap()
            .into_iter()
            .collect();
        println!("Nodes: {:?}", nodes);

        // 为每个节点启动一个任务
        for node_index in nodes {
            if let Some(node) = self.graph.node_weight_mut(node_index) {
                let node_type = node.node_type.clone();
                
                let node = node.clone();
                let handle = tokio::spawn(async move {
                    match node_type {
                        NodeType::DataSource => {
                            ExecutionGraph::run_data_source(&node).await;
                        },

                        NodeType::Indicator(indicator_type) => {
                            ExecutionGraph::run_indicator(&node).await;
                        },
                        NodeType::Condition => {
                            ExecutionGraph::run_condition(&node).await;
                        }
                        _ => {}
                    }
                });

                handles.push(handle);
            }
        }
    }

    pub async fn run_data_source(node: &Node) {
        let mut rng = StdRng::from_entropy();
        loop {
            // 生成随机数据
            let price = rng.gen_range(1.0..100.0);
            println!("Price: {:?}", price);
            
            // 发送到所有输出通道
            match node.sender.send(NodeData::Price(price)) {
                Ok(receiver_count) => {
                    println!(
                        "价格 {:?} 发送成功, 共有 {} 个接收者", 
                        price, 
                        receiver_count
                    );
                },
                Err(e) => {
                    println!(
                        "价格 {:?} 发送失败: 错误 = {:?}, 接收者数量 = {}", 
                        price,
                        e,
                        node.sender.receiver_count()
                    );
                }
            }
    
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub async fn run_indicator(
        node: &Node,
    ) {
        println!("开始运行指标: {:?}", node.node_type);
        // 遍历所有接收者
        // 将所有 receiver 转换为 stream
        let streams: Vec<_> = node.receivers.iter()
            .map(|rx| BroadcastStream::new(rx.resubscribe()))
            .collect();

        // 合并所有 stream
        let mut combined_stream = select_all(streams);

        while let Some(result) = combined_stream.next().await {
            match result {
                Ok(data) => {
                    if let NodeData::Price(price) = data {
                        println!("{}接收到价格: {:?}", node.id, price);
                        let result = match &node.node_type {
                            NodeType::Indicator(indicator_type) => {
                                match indicator_type.as_str() {
                                    "SMA12" => price + 5.0,
                                    "SMA24" => price + 10.0,
                                    _ => price,
                                }
                            }
                            _ => price,
                        };

                        if let Err(e) = node.sender.send(NodeData::Indicator(result)) {
                            println!("发送失败: {:?}", e);
                        }
                    }
                }
                Err(e) => println!("接收数据错误: {:?}", e),
            }
        }
    }

    pub async fn run_condition(node: &Node) {
        let mut sma12_value: Option<f64> = None;
        let mut sma24_value: Option<f64> = None;

        // 将所有 receiver 转换为 stream
        let streams: Vec<_> = node.receivers.iter()
            .map(|rx| BroadcastStream::new(rx.resubscribe()))
            .collect();

        // 合并所有 stream
        let mut combined_stream = select_all(streams);

        while let Some(result) = combined_stream.next().await {
            match result {
                Ok(data) => {
                    if let NodeData::Indicator(value) = data {
                        // 临时存储指标值
                        if sma12_value.is_none() {
                            sma12_value = Some(value);
                        } else if sma24_value.is_none() {
                            sma24_value = Some(value);
                        }
            
                        // 当两个值都收到时，进行比较
                        if let (Some(v1), Some(v2)) = (sma12_value, sma24_value) {
                            let signal = v1 > v2;
                            println!("SMA12: {:.2}, SMA24: {:.2}, Signal: {}", v1, v2, signal);
                            
                            // 重置值
                            sma12_value = None;
                            sma24_value = None;
                        }
                    }
                }
                Err(e) => println!("接收数据错误: {:?}", e),
            }
        }
    }

    pub fn topological_sort(&self) -> Vec<&Node> {
        petgraph::algo::toposort(&self.graph, None)
        .unwrap_or_default()
        .into_iter()
        .map(|index| &self.graph[index])
        .collect()
    }

    pub fn get_direct_inputs(&self, node_id: &str) -> Vec<&Node> {
        if let Some(&node_index) = self.node_indices.get(node_id) {
            self.graph.neighbors_directed(node_index, petgraph::Direction::Incoming)
                .map(|idx| &self.graph[idx])
                .collect()
        } else {
            vec![]
        }
    }

}
