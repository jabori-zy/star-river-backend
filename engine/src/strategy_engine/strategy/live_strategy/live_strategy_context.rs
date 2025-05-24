use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use types::custom_type::StrategyId;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::node::LiveNodeTrait;
use types::strategy::{TradeMode, LiveStrategyConfig};
use types::strategy::node_message::NodeMessage;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::position::Position;
use std::any::Any;
use database::query::position_query::PositionQuery;
use database::mutation::position_mutation::PositionMutation;
use types::position::PositionState;
use types::strategy::node_message::PositionMessage;
use super::live_strategy_function::sys_variable_function::SysVariableFunction;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_state_machine::LiveNodeRunState;
use types::cache::CacheKey;
use uuid::Uuid;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheMultiParams};
use event_center::response::cache_engine_response::CacheEngineResponse;
use utils::get_utc8_timestamp_millis;
use event_center::command::Command;
use event_center::strategy_event::{StrategyEvent, StrategyData};
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use tokio::sync::oneshot;
use types::strategy::node_command::{NodeCommandReceiver, NodeCommand};
use super::live_strategy_state_machine::LiveStrategyStateMachine;

#[derive(Debug)]
// 实盘策略上下文
pub struct LiveStrategyContext {
    pub strategy_id: i32,
    pub strategy_name: String, // 策略名称
    pub strategy_config: LiveStrategyConfig, // 策略配置
    pub cache_keys: Arc<RwLock<Vec<CacheKey>>>, // 缓存键
    pub graph: Graph<Box<dyn LiveNodeTrait>, (),  Directed>, // 策略的拓扑图
    pub node_indices: HashMap<String, NodeIndex>, // 节点索引
    pub event_publisher: EventPublisher, // 事件发布器
    pub event_receivers: Vec<EventReceiver>, // 事件接收器
    pub command_publisher: CommandPublisher, // 命令发布器
    pub command_receiver: Arc<Mutex<CommandReceiver>>, // 命令接收器
    pub cancel_token: CancellationToken, // 取消令牌
    pub state_machine: LiveStrategyStateMachine, // 策略状态机
    pub all_node_output_handles: Vec<NodeOutputHandle>, // 接收策略内所有节点的消息
    pub positions: Arc<RwLock<Vec<Position>>>, // 策略的所有持仓
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>, // 交易所引擎
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub registered_tasks: Arc<RwLock<HashMap<String, Uuid>>>, // 注册的任务 任务名称-> 任务id
    pub strategy_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 策略命令接收器
}


impl LiveStrategyContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // fn clone_box(&self) -> Box<dyn StrategyContext> {
    //     Box::new(self.clone())
    // }

    pub fn get_strategy_id(&self) -> i32 {
        self.strategy_id
    }

    pub fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }

    pub async fn get_cache_keys(&self) -> Vec<CacheKey> {
        self.cache_keys.read().await.clone()
    }

    pub fn get_state_machine(&self) -> LiveStrategyStateMachine {
        self.state_machine.clone()
    }

    pub fn set_state_machine(&mut self, state_machine: LiveStrategyStateMachine) {
        self.state_machine = state_machine;
    }

    pub fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle> {
        self.all_node_output_handles.clone()
    }


    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {
        &self.event_receivers
    }

    pub fn get_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.strategy_command_receiver.clone()
    }

    pub async fn handle_command(&mut self, command: NodeCommand) -> Result<(), String> {
        Ok(())
    }

    pub async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        // if let Event::Response(ResponseEvent::CacheEngine(CacheEngineResponse::GetCacheDataMulti(response))) = event {
        //     let strategy_data = StrategyData {
        //         strategy_id: self.strategy_id,
        //         data: response.cache_data,
        //         timestamp: get_utc8_timestamp_millis(),
        //     };
        //     let strategy_event = StrategyEvent::StrategyDataUpdate(strategy_data);
        //     let _ = self.event_publisher.publish(strategy_event.into());
        // }
        Ok(())
    }

    pub async fn handle_node_message(&mut self, message: NodeMessage) -> Result<(), String> {
        // tracing::debug!("策略: {:?} 收到来自节点消息: {:?}", self.get_strategy_name(), message);
        match message {
            NodeMessage::Position(position_message) => {
                match position_message {
                    // 仓位更新事件
                    PositionMessage::PositionUpdated(position) => {
                        // 更新持仓
                        self.positions.write().await.push(position);
                        // 更新系统变量
                        let sys_variable = SysVariableFunction::update_position_number(&self.database, self.strategy_id).await.unwrap();
                        tracing::info!("更新系统变量: {:?}", sys_variable);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    
}

impl LiveStrategyContext {

    // 拓扑排序
    pub fn topological_sort(&self) -> Vec<Box<dyn LiveNodeTrait>> {
        petgraph::algo::toposort(&self.graph, None)
        .unwrap_or_default()
        .into_iter()
        .map(|index| self.graph[index].clone())
        .collect()
    }

    // 启用策略的数据推送功能
    pub async fn enable_strategy_data_push(&mut self) {
        let command_publisher = self.command_publisher.clone();
        let event_publisher = self.event_publisher.clone();
        let strategy_id = self.strategy_id;
        let strategy_name = self.strategy_name.clone();
        let cache_keys = self.cache_keys.clone();

        let mut heartbeat = self.heartbeat.lock().await;
        let task_id = heartbeat.register_async_task(
            "启用策略数据推送".to_string(), 
            move || {
                let strategy_id = strategy_id;
                let strategy_name = strategy_name.clone();
                let cache_keys = cache_keys.clone();
                let command_publisher = command_publisher.clone();
                let event_publisher = event_publisher.clone();
                async move {
                    Self::get_strategy_data(strategy_id, strategy_name, cache_keys, command_publisher, event_publisher).await;
                }
            },
            5
        ).await;
        self.registered_tasks.write().await.insert("push_strategy_data".to_string(), task_id);
        tracing::debug!("任务注册成功，当前任务列表：{:?}", self.registered_tasks.read().await);

    }

    pub async fn disable_strategy_data_push(&mut self) {
        let task_id = self.registered_tasks.write().await.remove("push_strategy_data");
        
        if let Some(task_id) = task_id {
            let mut heartbeat = self.heartbeat.lock().await;
            heartbeat.unregister_task(task_id).await.unwrap();
            tracing::debug!("任务取消成功，当前任务列表：{:?}", self.registered_tasks.read().await);
        }
    }

    async fn get_strategy_data(
        strategy_id: StrategyId,
        strategy_name: String,
        cache_keys: Arc<RwLock<Vec<CacheKey>>>,
        command_publisher: CommandPublisher,
        event_publisher: EventPublisher,
    ) {
        let cache_keys_clone = cache_keys.read().await.clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheMultiParams {
            strategy_id: strategy_id,
            cache_keys: cache_keys_clone,
            index: None,
            limit: Some(1), // 只获取最新的一条数据
            sender: strategy_name,
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let get_cache_multi_command = Command::CacheEngine(CacheEngineCommand::GetCacheMulti(params));
        command_publisher.send(get_cache_multi_command).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            let cache_engine_response = CacheEngineResponse::try_from(response);
            if let Ok(cache_engine_response) = cache_engine_response {
                match cache_engine_response {
                    CacheEngineResponse::GetCacheDataMulti(response) => {
                        let strategy_data = StrategyData {
                            strategy_id: strategy_id,
                            data: response.cache_data,
                            timestamp: get_utc8_timestamp_millis(),
                        };
                        let strategy_event = StrategyEvent::LiveStrategyDataUpdate(strategy_data);
                        event_publisher.publish(strategy_event.into()).await.unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    

    pub async fn wait_for_all_nodes_running(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        loop {

            let mut all_running = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                let run_state = node.get_run_state().await;
                if run_state != LiveNodeRunState::Running {
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
                
    

    pub async fn wait_for_all_nodes_stopped(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        loop {
            let mut all_stopped = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                let run_state = node.get_run_state().await;
                if run_state != LiveNodeRunState::Stopped {
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


    pub async fn init_node(&self, node: Box<dyn LiveNodeTrait>) -> Result<(), String> {
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
        match tokio::time::timeout(Duration::from_secs(30), node_handle).await {
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
            if run_state == LiveNodeRunState::Ready {
                tracing::debug!("节点 {} 已进入Ready状态", node_id);
                // 节点初始化间隔
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Ready状态", node_id))
    }

    // 添加一个新的辅助方法
    pub async fn start_node(&self, node: Box<dyn LiveNodeTrait>) -> Result<(), String> {
        
        
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
            if run_state == LiveNodeRunState::Running {
                tracing::debug!("节点 {} 已进入Running状态", node_id);
                // 节点启动间隔
                // tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Running状态", node_id))
    }

    pub async fn stop_node(&self, node: Box<dyn LiveNodeTrait>) -> Result<(), String> {
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
            if run_state == LiveNodeRunState::Stopped {
                tracing::debug!("节点 {} 已进入Stopped状态", node_id);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Stopped状态", node_id))


    }
    

    // 获取策略的所有持仓
    pub async fn load_all_positions(&mut self) {
        let positions = PositionQuery::get_all_positions_by_strategy_id(&self.database, self.strategy_id).await.unwrap();
        self.positions.write().await.extend(positions);
    }

    // 监控持仓
    pub async fn monitor_positions(&mut self) {
        let positions = self.positions.clone();
        let exchange_engine = self.exchange_engine.clone();
        let database = self.database.clone();
        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            "监控持仓".to_string(),
            move || {
                let positions = positions.clone();
                let exchange_engine = exchange_engine.clone();
                let database = database.clone();
                async move {
                    Self::process_positions(
                        positions,
                        exchange_engine,
                        database
                    ).await
                }
            },
            10
        ).await;
    }

    // 处理仓位
    async fn process_positions(
        positions: Arc<RwLock<Vec<Position>>>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
    ) {
        let positions_clone = {
            let positions = positions.read().await;
            positions.clone()
        };

        // 如果hashmap为空，则直接返回
        if positions_clone.is_empty() {
            return;
        }

        // 遍历持仓, 获取下标和持仓
        for (index, position) in positions_clone.iter().enumerate() {
            // 获取交易所的上下文
            let exchange_engine_guard = exchange_engine.lock().await;
            // 获取交易所对象
            let exchange = exchange_engine_guard.get_exchange(&position.account_id).await;
            match exchange {
                Ok(exchange) => {
                    // 获取持仓信息
                    let latest_position = exchange.get_latest_position(position).await;
                    match latest_position {
                        Ok(position) => {
                            // 更新列表中的持仓
                            positions.write().await[index] = position.clone();
                            // 更新持仓到数据库
                            PositionMutation::update_position(
                                &database,
                                position.clone()
                            ).await.unwrap();

                            // tracing::info!("未平仓利润: {:?}", position.unrealized_profit);


                        }
                        Err(e) => {
                            // tracing::error!("获取最新持仓失败: {:?}", e);
                        }
                    }
                    
                }
                Err(_) => {
                    tracing::warn!("仓位已关闭: {:?}", position.position_id);
                    PositionMutation::update_position_state(
                        &database,
                        position.position_id,
                        PositionState::Closed
                    ).await.unwrap();
                }
            }
        }

        
    }



}