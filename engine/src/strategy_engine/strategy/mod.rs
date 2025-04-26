pub mod add_node;
pub mod add_start_node;
pub mod add_live_data_node;
pub mod add_if_else_node;
pub mod add_indicator_node;
pub mod add_order_node;
pub mod strategy_state_manager;

use crate::*;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::{broadcast, watch};
use event_center::{Event, EventPublisher};
use serde_json::Value;
use tokio::join;
use tokio::sync::mpsc;
use event_center::strategy_event::StrategyEvent;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use std::thread::JoinHandle;
use super::node::NodeTrait;
use super::strategy::strategy_state_manager::{StrategyStateMachine, StrategyRunState, StrategyStateTransitionEvent, StrategyStateAction};
use super::node::node_types::NodeMessageReceiver;
use super::node::node_types::NodeRunState;
use types::strategy::{TradeMode, StrategyConfig};
use database::entities::strategy_config::Model as StrategyConfigModel;


#[derive(Debug)]
// 策略图
pub struct Strategy {
    pub strategy_id: i32,
    pub strategy_name: String, // 策略名称
    pub trading_mode: TradeMode, // 交易模式
    pub config: StrategyConfig, // 策略配置
    pub graph: Graph<Box<dyn NodeTrait>, (),  Directed>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub event_publisher: EventPublisher,
    pub response_event_receiver: broadcast::Receiver<Event>,
    pub enable_event_publish: bool,
    pub cancel_token: CancellationToken,
    pub state_manager: StrategyStateMachine,
}


impl Clone for Strategy {
    fn clone(&self) -> Self {
        Self {
            strategy_id: self.strategy_id,
            strategy_name: self.strategy_name.clone(),
            trading_mode: self.trading_mode.clone(),
            config: self.config.clone(),
            graph: self.graph.clone(),
            node_indices: self.node_indices.clone(),
            event_publisher: self.event_publisher.clone(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            enable_event_publish: self.enable_event_publish,
            cancel_token: self.cancel_token.clone(),
            state_manager: self.state_manager.clone(),
        }
    }
}



impl Strategy {
    pub async fn new(
        strategy_config: StrategyConfigModel, 
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>, 
        response_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let mut graph = Graph::new();
        let mut node_indices = HashMap::new();
        let mut global_config = StrategyConfig::default();

        let strategy_id = strategy_config.id;
        let strategy_name = strategy_config.name;
        let trade_mode = match strategy_config.trade_mode.as_str() {
            "backtest" => TradeMode::Backtest,
            "simulated" => TradeMode::Simulated,
            "live" => TradeMode::Live,
            _ => TradeMode::Backtest,
        };

        // 当策略创建时，状态为 Created
        let cancel_token = CancellationToken::new();


        match strategy_config.config {
            Some(config) => {
                if let Ok(config) = serde_json::from_str::<StrategyConfig>(&config.to_string()) {
                    global_config = config;
                } else {
                    tracing::error!("策略配置解析失败");
                }
            }
            None => {
                tracing::warn!("策略配置为空");
            }
        }

        
        if let Some(nodes_str) = strategy_config.nodes {
            if let Ok(nodes) = serde_json::from_str::<Vec<Value>>(&nodes_str.to_string()) {
                for node_config in nodes {
                    tracing::debug!("添加节点: {:?}", node_config);
                    Self::add_node(
                        &mut graph, 
                        &mut node_indices, 
                        trade_mode.clone(),
                        &node_config, 
                        event_publisher.clone(), 
                        market_event_receiver.resubscribe(), 
                        response_event_receiver.resubscribe(),
                    ).await;

                }
            }
        }
        // 添加边
        if let Some(edges_str) = strategy_config.edges {
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
            strategy_name: strategy_name.clone(),
            trading_mode: trade_mode,
            config: global_config,
            graph,
            node_indices,
            event_publisher,
            response_event_receiver,
            enable_event_publish: false,
            cancel_token,
            state_manager: StrategyStateMachine::new(strategy_id, strategy_name, StrategyRunState::Created),
        }
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
            
            tracing::debug!("添加边: {:?} -> {:?}, 源节点handle = {}", from_node_id, to_node_id, from_handle_id);
            // 先获取源节点的发送者
            let sender = graph.node_weight(source).unwrap().get_message_sender(from_handle_id.to_string()).await;
            
            tracing::debug!("{}: sender: {:?}", from_handle_id, sender);
            // 增加源节点的出口连接数
            graph.node_weight_mut(source).unwrap().add_output_handle_connect_count(from_handle_id.to_string()).await;
            // tracing::debug!("sender: {:?}", sender);

            if let Some(target_node) = graph.node_weight_mut(target) {
                let receiver = sender.subscribe();
                // 获取接收者数量
                let message_receivers = target_node.get_message_receivers().await;
                tracing::debug!("{:?} 添加了一个接收者", target_node.get_node_name().await);
                target_node.add_message_receiver(NodeMessageReceiver::new(from_node_id.to_string(), receiver)).await;
                tracing::debug!("{}: 添加了一个接收者: {:?}", target_node.get_node_name().await, message_receivers);
                target_node.add_from_node_id(from_node_id.to_string()).await;
            }
            // tracing::debug!("添加边: {:?} -> {:?}", from_node_id, to_node_id);
            graph.add_edge(source, target, ());
        }
        

    }

    // 拓扑排序
    pub fn topological_sort(&self) -> Vec<&Box<dyn NodeTrait>> {
        petgraph::algo::toposort(&self.graph, None)
        .unwrap_or_default()
        .into_iter()
        .map(|index| &self.graph[index])
        .collect()
    }


    

    // 启用策略的事件发布功能
    pub async fn enable_strategy_event_push(&mut self) {
        self.enable_event_publish = true;
        // 遍历所有节点，设置 enable_event_publish 为 true
        for node in self.graph.node_weights_mut() {
            node.enable_node_event_push().await.unwrap();
        }
    }

    pub async fn disable_event_push(&mut self) {
        self.enable_event_publish = false;
        // 遍历所有节点，设置 enable_event_publish 为 false
        for node in self.graph.node_weights_mut() {
            node.disable_node_event_push().await.unwrap();
        }
    }

    
    pub async fn update_strategy_state(&mut self, event: StrategyStateTransitionEvent) {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let strategy_id = self.strategy_id;
        let strategy_name = self.strategy_name.clone();

        let (transition_result, state_manager) = {
            let mut state_manager = self.state_manager.clone();
            let transition_result = state_manager.transition(event).unwrap();
            (transition_result, state_manager)
        };

        tracing::info!("需要执行的动作: {:?}", transition_result.actions);
        for action in transition_result.actions.clone() {
            match action {
                StrategyStateAction::InitNode => {
                    tracing::info!("++++++++++++++++++++++++++++++++++++++");
                    tracing::info!("{}: 开始初始化节点", self.strategy_name);
                    let nodes = self.topological_sort();
                    
                    let mut all_nodes_initialized = true;

                    for node in nodes {
                        let mut node = node.clone();
                        
                        if let Err(e) = self.init_node(&mut node).await {
                            tracing::error!("{}", e);
                            all_nodes_initialized = false;
                            break;
                        }
                    }

                    if all_nodes_initialized {
                        tracing::info!("{}: 所有节点已成功初始化", self.strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点初始化失败，策略无法正常运行", self.strategy_name);
                    }
                }

                StrategyStateAction::StartNode => {
                    tracing::info!("++++++++++++++++++++++++++++++++++++++");
                    tracing::info!("{}: 开始启动节点", self.strategy_name);
                    let nodes = self.topological_sort();
                    
                    let mut all_nodes_started = true;

                    for node in nodes {
                        let mut node = node.clone();
                        
                        if let Err(e) = self.start_node(&mut node).await {
                            tracing::error!("{}", e);
                            all_nodes_started = false;
                            break;
                        }
                    }

                    if all_nodes_started {
                        tracing::info!("{}: 所有节点已成功启动", self.strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点启动失败，策略无法正常运行", self.strategy_name);
                    }
                }


                StrategyStateAction::StopNode => {
                    tracing::info!("++++++++++++++++++++++++++++++++++++++");
                    tracing::info!("{}: 开始停止节点", self.strategy_name);
                    let nodes = self.topological_sort();
                    
                    let mut all_nodes_stopped = true;

                    for node in nodes {
                        let mut node = node.clone();
                        
                        if let Err(e) = self.stop_node(&mut node).await {
                            tracing::error!("{}", e);
                            all_nodes_stopped = false;
                            break;
                        }
                    }

                    if all_nodes_stopped {
                        tracing::info!("{}: 所有节点已成功停止", self.strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点停止失败，策略无法正常运行", self.strategy_name);
                    }
                }
                
                StrategyStateAction::LogTransition => {
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", self.strategy_name, self.state_manager.current_state(), transition_result.new_state);
                }
                _ => {}
            }


            // 更新策略状态
            self.state_manager = state_manager.clone();

            
        }
        
    }
    
    pub async fn start_strategy(&mut self) -> Result<(), String> {
        tracing::info!("启动策略: {}", self.strategy_name);
        // 获取当前状态
        let current_state = self.state_manager.current_state();
        // 如果当前状态为 Running，则不进行操作
        if current_state != StrategyRunState::Ready {
            tracing::info!("策略未处于Ready状态, 不能启动: {}", self.strategy_name);
            return Ok(());
        }
        // 策略发送启动信号

        tracing::info!("{}: 发送启动策略信号", self.strategy_name);
        tracing::info!("等待所有节点启动...");
        self.update_strategy_state(StrategyStateTransitionEvent::Start).await;

        // todo: 这里需要循环检测每个节点是否启动成功，也就是检测每个节点是否都是running状态
        // 如果所有节点都启动成功，则更新策略状态为Running
        // 如果有一个节点启动失败，则返回失败
        let all_running = self.wait_for_all_nodes_running(10).await.unwrap();
        if all_running {
            self.update_strategy_state(StrategyStateTransitionEvent::StartComplete).await;
            Ok(())
        } else {
            Err("等待节点启动超时".to_string())
        }
    }

    pub async fn stop_strategy(&mut self) -> Result<(), String> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        if self.state_manager.current_state() == StrategyRunState::Stopping {
            tracing::info!("策略{}已停止", self.strategy_name);
            return Ok(());
        }
        tracing::info!("等待所有节点停止...");
        self.update_strategy_state(StrategyStateTransitionEvent::Stop).await;

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = self.wait_for_all_nodes_stopped(10).await.unwrap();
        if all_stopped {
            self.update_strategy_state(StrategyStateTransitionEvent::StopComplete).await;
            Ok(())
        } else {
            Err("等待节点停止超时".to_string())
        }
    }

    async fn wait_for_all_nodes_running(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        loop {

            let mut all_running = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                let run_state = node.get_run_state().await;
                if run_state != NodeRunState::Running {
                    all_running = false;
                    break;
                }
            }
            
            // 如果所有节点都已启动，返回成功
            if all_running {
                tracing::info!("所有节点已启动，共耗时{}ms", start_time.elapsed().as_millis());
                return Ok(true);
            }
            
            // 检查是否超时
            if start_time.elapsed() > timeout {
                tracing::warn!("等待节点启动超时，已等待{}秒", timeout_secs);
                return Ok(false);
            }
            
            // 短暂休眠后再次检查
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
                
    

    async fn wait_for_all_nodes_stopped(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        loop {
            let mut all_stopped = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                let run_state = node.get_run_state().await;
                if run_state != NodeRunState::Stopped {
                    all_stopped = false;
                    break;
                }
            }
            
            // 如果所有节点都已停止，返回成功
            if all_stopped {
                tracing::info!("所有节点已停止，共耗时{}ms", start_time.elapsed().as_millis());
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
        tracing::info!("{}: 开始初始化策略", self.strategy_name);

        // created => initializing
        self.update_strategy_state(StrategyStateTransitionEvent::Initialize).await;

        // 
        // initializing => ready
        tracing::info!("{}: 初始化完成", self.strategy_name);
        self.update_strategy_state(StrategyStateTransitionEvent::InitializeComplete).await;

        // // 更新策略状态为 Ready
        // self.state_tx.send(StrategySignal::Start).map_err(|_| {"更新状态失败".to_string()}).unwrap();
        // tracing::info!("{} 初始化完成。当前状态：{:?}", self.strategy_name, self.state_tx.borrow().clone());
        Ok(())
    }

    async fn init_node(&self, node: &mut Box<dyn NodeTrait>) -> Result<(), String> {
        let mut node_clone = node.clone();

        let node_handle = tokio::spawn(async move {
            let node_name = node_clone.get_node_name().await;
            if let Err(e) = node_clone.init().await {
                tracing::error!("{} 节点初始化失败: {}", node_name, e);
                return Err(format!("节点初始化失败: {}", e));
            }
            Ok(())
        });


        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;
        
        // 等待节点初始化完成
        match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("节点 {} 初始化任务失败: {}", node_name, e));
                }
                
                if let Ok(Err(e)) = result {
                    return Err(format!("节点 {} 初始化过程中出错: {}", node_name, e));
                }
            }
            Err(_) => {
                return Err(format!("节点 {} 初始化超时", node_id));
            }
        }
        // 等待节点进入Running状态
        let mut retry_count = 0;
        let max_retries = 20;
        
        while retry_count < max_retries {
            let run_state = node.get_run_state().await;
            if run_state == NodeRunState::Ready {
                tracing::debug!("节点 {} 已进入Ready状态", node_id);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Ready状态", node_id))
    }

    // 添加一个新的辅助方法
    async fn start_node(&self, node: &mut Box<dyn NodeTrait>) -> Result<(), String> {
        
        
        // 启动节点
        let mut node_clone = node.clone();
        
        let node_handle = tokio::spawn(async move {
            let node_name = node_clone.get_node_name().await;
            if let Err(e) = node_clone.start().await {
                tracing::error!("{} 节点启动失败: {}", node_name, e);
                return Err(format!("节点启动失败: {}", e));
            }
            Ok(())
        });

        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;
        
        
        // 等待节点启动完成
        match tokio::time::timeout(Duration::from_secs(30), node_handle).await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("节点 {} 启动任务失败: {}", node_name, e));
                }
                
                if let Ok(Err(e)) = result {
                    return Err(format!("节点 {} 启动过程中出错: {}", node_name, e));
                }
            }
            Err(_) => {
                return Err(format!("节点 {} 启动超时", node_id));
            }
        }
        
        // 等待节点进入Running状态
        let mut retry_count = 0;
        let max_retries = 50;
        
        while retry_count < max_retries {
            let run_state = node.get_run_state().await;
            if run_state == NodeRunState::Running {
                tracing::debug!("节点 {} 已进入Running状态", node_id);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Running状态", node_id))
    }

    async fn stop_node(&self, node: &mut Box<dyn NodeTrait>) -> Result<(), String> {
        // 启动节点
        let mut node_clone = node.clone();
        
        let node_handle = tokio::spawn(async move {
            let node_name = node_clone.get_node_name().await;
            if let Err(e) = node_clone.stop().await {
                tracing::error!("{} 节点停止失败: {}", node_name, e);
                return Err(format!("节点停止失败: {}", e));
            }
            Ok(())
        });

        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;
        
        
        // 等待节点启动完成
        match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("节点 {} 停止任务失败: {}", node_name, e));
                }
                
                if let Ok(Err(e)) = result {
                    return Err(format!("节点 {} 停止过程中出错: {}", node_name, e));
                }
            }
            Err(_) => {
                return Err(format!("节点 {} 停止超时", node_id));
            }
        }
        
        // 等待节点进入Running状态
        let mut retry_count = 0;
        let max_retries = 20;
        
        while retry_count < max_retries {
            let run_state = node.get_run_state().await;
            if run_state == NodeRunState::Stopped {
                tracing::debug!("节点 {} 已进入Stopped状态", node_id);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Stopped状态", node_id))


    }

}

    




