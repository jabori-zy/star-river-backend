pub mod add_node;
pub mod add_start_node;
pub mod add_live_data_node;
pub mod add_if_else_node;
pub mod add_indicator_node;

use crate::*;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::{broadcast, watch};
use crate::node::buy_node::BuyNode;
use event_center::{Event, EventPublisher};
use database::entities::strategy_info::Model as StrategyInfo;
use serde_json::Value;
use tokio::join;
use tokio::sync::mpsc;
use event_center::strategy_event::StrategyEvent;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use std::thread::JoinHandle;
use crate::node::NodeTrait;


#[derive(Debug, Clone, PartialEq)]
pub enum StrategyState {
    Created,
    Ready,
    Running,
    Stopping,
    Stopped,
}

#[derive(Debug)]
// 策略图
pub struct Strategy {
    pub strategy_id: i32,
    pub strategy_name: String,
    pub graph: Graph<Box<dyn NodeTrait>, (),  Directed>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub event_publisher: EventPublisher,
    pub response_event_receiver: broadcast::Receiver<Event>,
    pub strategy_event_receiver: broadcast::Receiver<Event>,
    pub enable_event_publish: bool,
    pub state_tx: watch::Sender<StrategyState>,
    pub node_handle_list: HashMap<String, tokio::task::JoinHandle<()>>, // 节点id -> 节点handle
    pub cancel_token: CancellationToken,
}



impl Strategy {
    pub async fn new(
        strategy_info: StrategyInfo, 
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>, 
        response_event_receiver: broadcast::Receiver<Event>,
        strategy_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();

        let strategy_id = strategy_info.id;
        let strategy_name = strategy_info.name;

        // 当策略创建时，状态为 Created
        let (state_tx, state_rx) = watch::channel(StrategyState::Created);
        let cancel_token = CancellationToken::new();
        
        if let Some(nodes_str) = strategy_info.nodes {
            if let Ok(nodes) = serde_json::from_str::<Vec<Value>>(&nodes_str.to_string()) {
                for node_config in nodes {
                    // tracing::debug!("添加节点: {:?}", node_config);
                    Self::add_node(
                        &mut graph, 
                        &mut node_indices, 
                        &node_config, 
                        event_publisher.clone(), 
                        market_event_receiver.resubscribe(), 
                        response_event_receiver.resubscribe(),
                        state_rx.clone(),
                        cancel_token.clone()

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
            strategy_id,
            strategy_name,
            graph,
            node_indices,
            event_publisher,
            response_event_receiver,
            strategy_event_receiver,
            enable_event_publish: false,
            state_tx,
            node_handle_list: HashMap::new(),
            cancel_token,
        }
    }






    pub async fn add_buy_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_id: String,
        node_name: String,
        event_publisher: EventPublisher
    ) {

        let node = Box::new(BuyNode::new(node_id.clone(), node_name.clone(), event_publisher).init_node().await);
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
            // 增加源节点的出口连接数
            graph.node_weight_mut(source).unwrap().add_node_output_handle_connect_count(from_handle_id.to_string()).await;
            // tracing::debug!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let receiver_count = sender.receiver_count();
                // tracing::debug!("{:?} 添加了一个接收者, 接收者数量 = {}", target_node.get_node_name().await, receiver_count);
                target_node.add_message_receiver(receiver);
                target_node.add_from_node_id(from_node_id.to_string());
            }
            // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
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

    async fn listen_external_events(&self, internal_tx: mpsc::Sender<Event>) {
        let mut strategy_event_receiver = self.strategy_event_receiver.resubscribe();
        tokio::spawn(async move {
            loop {
                let event = strategy_event_receiver.recv().await.unwrap();
                let _ = internal_tx.send(event).await;
            }
        });
    }

    // 处理接收到的事件
    async fn handle_external_events(mut internal_rx: mpsc::Receiver<Event>) {
        tokio::spawn(async move {
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Strategy(strategy_event) => {
                    Self::handle_strategy_event(strategy_event).await;
                }
                _ => {}
            }
            }
        });
    }

    async fn handle_strategy_event(_strategy_event: StrategyEvent) {
        // tracing::debug!("接收到策略事件: {:?}", strategy_event);
    }

    

    // 启用策略的事件发布功能
    pub async fn enable_strategy_event_publish(&mut self) {
        self.enable_event_publish = true;
        // 遍历所有节点，设置 enable_event_publish 为 true
        for node in self.graph.node_weights_mut() {
            node.enable_node_event_publish().await;
        }
    }

    pub fn disable_event_publish(&mut self) {
        self.enable_event_publish = false;
    }

    pub async fn start_strategy(&mut self) -> Result<(), String> {
        tracing::info!("启动策略: {}", self.strategy_name);
        // 获取当前状态
        let current_state = self.state_tx.borrow().clone();
        // 如果当前状态为 Running，则不进行操作
        if current_state == StrategyState::Running {
            tracing::info!("策略{}正在运行", self.strategy_name);
            return Ok(());
        }
        // 策略发送启动信号
        if current_state == StrategyState::Ready {
            tracing::info!("{}: 发送启动策略信号", self.strategy_name);
            self.state_tx.send(StrategyState::Running).map_err(|_| {"更新启动状态失败".to_string()})?;
        }
        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<StrategyState, String> {
        // 获取当前状态
        let current_state = self.state_tx.borrow().clone();
        // 如果策略当前状态为 Stopped，则不进行操作
        if current_state == StrategyState::Stopped {
            tracing::info!("策略{}已停止", self.strategy_name);
            return Ok(StrategyState::Stopped);
        }
        // 策略发送停止信号
        self.state_tx.send(StrategyState::Stopping).map_err(|_| {"更新停止状态失败".to_string()})?;

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = self.wait_for_all_nodes_stopped(10).await.unwrap();
        if all_stopped {
            Ok(StrategyState::Stopped)
        } else {
            Err("等待节点停止超时".to_string())
        }
    }

    async fn wait_for_all_nodes_stopped(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        loop {
            tracing::info!("等待节点停止...");
            let mut all_stopped = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                if node.get_node_run_state().await != NodeRunState::Stopped {
                    all_stopped = false;
                    break;
                }
            }
            
            // 如果所有节点都已停止，返回成功
            if all_stopped {
                tracing::info!("所有节点已停止，共耗时{}秒", start_time.elapsed().as_secs());
                return Ok(true);
            }
            
            // 检查是否超时
            if start_time.elapsed() > timeout {
                tracing::warn!("等待节点停止超时，已等待{}秒", timeout_secs);
                return Ok(false);
            }
            
            // 短暂休眠后再次检查
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }


    // 设置策略
    pub async fn init_strategy (&mut self) -> Result<(), String> {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<>(100);
        // 监听策略事件
        self.listen_external_events(internal_tx).await;
        // 处理接收到的事件
        Self::handle_external_events(internal_rx).await;

        let nodes = self.topological_sort();
        // let mut handles = HashMap::new();
        // 循环启动每一个节点
        for node in nodes {
           
            let mut node = node.clone();
            let node_id = node.get_node_id().await;
            let node_name = node.get_node_name().await;
            let node_name_clone = node_name.clone();
            let node_handle = tokio::spawn(async move {
                tokio::select! {
                    node_setup_result = node.init() => {
                        // 如果初始化失败，则打印失败信息
                        if let Err(e) = node_setup_result {
                            tracing::error!("{} 节点初始化失败: {}", node_name, e);
                        }
                    },
                }
                
            });
            
            match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
                Ok(result) => {
                    result.map_err(|e| format!("节点 {} 设置任务失败: {}", node_name_clone, e))?;
                }
                Err(_) => {
                    tracing::error!("节点 {} 设置超时", node_id);
                }
            }
        }
        // 更新策略状态为 Ready
        self.state_tx.send(StrategyState::Ready).map_err(|_| {"更新状态失败".to_string()}).unwrap();
        tracing::info!("{} 初始化完成。当前状态：{:?}", self.strategy_name, self.state_tx.borrow().clone());
        Ok(())
    }
}

    




