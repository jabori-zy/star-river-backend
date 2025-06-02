use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use types::custom_type::StrategyId;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_state_machine::*;
use types::strategy::{TradeMode, LiveStrategyConfig};
use types::strategy::node_message::NodeMessage;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::position::Position;
use database::query::position_query::PositionQuery;
use database::mutation::position_mutation::PositionMutation;
use types::position::PositionState;
use types::strategy::node_message::PositionMessage;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_state_machine::BacktestNodeRunState;
use types::cache::CacheKey;
use uuid::Uuid;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheMultiParams};
use event_center::response::cache_engine_response::CacheEngineResponse;
use event_center::response::Response;
use utils::get_utc8_timestamp_millis;
use event_center::command::Command;
use event_center::strategy_event::{StrategyEvent, BacktestStrategyData};
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use tokio::sync::oneshot;
use types::strategy::BacktestStrategyConfig;
use types::strategy::node_command::{NodeCommandReceiver, NodeCommand, StrategyCommand};
use types::strategy::node_response::{GetStrategyCacheKeysResponse, StrategyResponse};
use tracing::instrument;
use event_center::command::cache_engine_command::GetCacheLengthMultiParams;
use crate::strategy_engine::node::node_types::NodeType;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;

#[derive(Debug)]
// 实盘策略上下文
pub struct BacktestStrategyContext {
    pub strategy_id: i32,
    pub strategy_name: String, // 策略名称
    pub strategy_config: BacktestStrategyConfig, // 策略配置
    pub cache_keys: Arc<RwLock<Vec<CacheKey>>>, // 缓存键
    pub cache_lengths: HashMap<CacheKey, u32>, // 缓存长度
    pub graph: Graph<Box<dyn BacktestNodeTrait>, (),  Directed>, // 策略的拓扑图
    pub node_indices: HashMap<String, NodeIndex>, // 节点索引
    pub event_publisher: EventPublisher, // 事件发布器
    pub event_receivers: Vec<EventReceiver>, // 事件接收器
    pub command_publisher: CommandPublisher, // 命令发布器
    pub command_receiver: Arc<Mutex<CommandReceiver>>, // 命令接收器
    pub cancel_token: CancellationToken, // 取消令牌
    pub state_machine: BacktestStrategyStateMachine, // 策略状态机
    pub all_node_output_handles: Vec<NodeOutputHandle>, // 接收策略内所有节点的消息
    pub positions: Arc<RwLock<Vec<Position>>>, // 策略的所有持仓
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>, // 交易所引擎
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub registered_tasks: Arc<RwLock<HashMap<String, Uuid>>>, // 注册的任务 任务名称-> 任务id
    pub strategy_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 策略命令接收器
    pub signal_count: Arc<RwLock<u32>>, // 信号计数
    pub played_signal_count: Arc<RwLock<u32>>, // 已发送的信号计数
    pub is_playing: Arc<RwLock<bool>>, // 是否正在播放
    pub initial_play_speed: Arc<RwLock<u32>>, // 初始播放速度 （从策略配置中加载）
    pub cancel_play_token: CancellationToken, // 取消播放令牌
}


impl BacktestStrategyContext {
    pub fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }

    async fn get_cache_keys(&self) -> Vec<CacheKey> {
        self.cache_keys.read().await.clone()
    }

    pub fn set_state_machine(&mut self, state_machine: BacktestStrategyStateMachine) {
        self.state_machine = state_machine;
    }

    pub fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle> {
        self.all_node_output_handles.clone()
    }


    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {
        &self.event_receivers
    }

    pub fn get_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.strategy_command_receiver.clone()
    }

    pub async fn handle_command(&mut self, command: NodeCommand) -> Result<(), String> {
        // tracing::info!("{}: 收到命令: {:?}", self.get_strategy_name(), command);
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
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
        // match message {
        //     NodeMessage::Position(position_message) => {
        //         match position_message {
        //             // 仓位更新事件
        //             PositionMessage::PositionUpdated(position) => {
        //                 // 更新持仓
        //                 self.positions.write().await.push(position);
        //                 // 更新系统变量
        //                 let sys_variable = SysVariableFunction::update_position_number(&self.database, self.strategy_id).await.unwrap();
        //                 tracing::info!("更新系统变量: {:?}", sys_variable);
        //             }
        //             _ => {}
        //         }
        //     }
        //     _ => {}
        // }
        Ok(())
    }
    
}

impl BacktestStrategyContext {

    // 拓扑排序
    pub fn topological_sort(&self) -> Vec<Box<dyn BacktestNodeTrait>> {
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
            // let cache_engine_response = CacheEngineResponse::try_from(response);
            // if let Ok(cache_engine_response) = cache_engine_response {
            //     match cache_engine_response {
            //         CacheEngineResponse::GetCacheDataMulti(response) => {
            //             let strategy_data = BacktestStrategyData {
            //                 strategy_id: strategy_id,
            //                 cache_key: response.cache_key.get_key(),
            //                 data: response.cache_data,
            //                 timestamp: get_utc8_timestamp_millis(),
            //             };
            //             let strategy_event = StrategyEvent::BacktestStrategyDataUpdate(strategy_data);
            //             event_publisher.publish(strategy_event.into()).await.unwrap();
            //         }
            //         _ => {}
            //     }
            // }
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
                if run_state != BacktestNodeRunState::Stopped {
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


    pub async fn init_node(&self, node: Box<dyn BacktestNodeTrait>) -> Result<(), String> {
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
            if run_state == BacktestNodeRunState::Ready {
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

    
    

    pub async fn stop_node(&self, node: Box<dyn BacktestNodeTrait>) -> Result<(), String> {
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
            if run_state == BacktestNodeRunState::Stopped {
                tracing::debug!("节点 {} 已进入Stopped状态", node_id);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Stopped状态", node_id))


    }

    #[instrument(skip(self))]
    // 获取所有k线缓存中的最小长度
    pub async fn get_cache_length(&self) -> Result<HashMap<CacheKey, u32>, String> {
        
        // 过滤出k线缓存key
        let kline_cache_keys = self.cache_keys
            .read()
            .await
            .iter()
            .filter(|cache_key| matches!(cache_key, CacheKey::BacktestKline(_)))
            .map(|cache_key| cache_key.clone())
            .collect();
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_length_params = GetCacheLengthMultiParams {
            strategy_id: self.strategy_id,
            cache_keys: kline_cache_keys,
            timestamp: get_utc8_timestamp_millis(),
            sender: self.strategy_name.clone(),
            responder: resp_tx
        };
        let cache_engine_command = CacheEngineCommand::GetCacheLengthMulti(get_cache_length_params);
        // 向缓存引擎发送命令
        self.command_publisher.send(cache_engine_command.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            let cache_engine_response = CacheEngineResponse::try_from(response);
            if let Ok(cache_engine_response) = cache_engine_response {
                match cache_engine_response {
                    CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response) => {
                        tracing::info!(cache_lengths = ?get_cache_length_multi_response.cache_length, "Get cache length successfully!");
                        Ok(get_cache_length_multi_response.cache_length)
                    }
                    _ => Err("get cache length multi failed".to_string())
                }
            } else {
                Err("try from response failed".to_string())
            }

        } else {
            Err("get cache length multi failed".to_string())
        }

    }

    // 初始化信号计数
    #[instrument(skip(self))]
    pub async fn get_signal_count(&mut self) -> Result<u32, String> {
        // 初始化信号计数
        let min_cache_length = self.cache_lengths.values().min().cloned().unwrap_or(0);
        Ok(min_cache_length)
    }

    // 播放k线
    pub async fn play(&self) {
        // 判断播放状态是否为true
        if *self.is_playing.read().await {
            tracing::warn!("{}: 正在播放，无需重复播放", self.strategy_name.clone());
            return;
        }
        // 播放状态为true
        *self.is_playing.write().await = true;

        // 获取开始节点的索引
        let start_node_index = self.node_indices.get("start_node").unwrap();
        let node = self.graph.node_weight(*start_node_index).unwrap().clone();

        let played_signal_count = self.played_signal_count.clone();
        let signal_count = self.signal_count.clone();
        let is_playing = self.is_playing.clone();
        let initial_play_speed = self.initial_play_speed.clone();
        
        let strategy_name = self.strategy_name.clone();
        let child_cancel_play_token = self.cancel_play_token.child_token();
        
        tokio::spawn(async move {
            let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();

            loop {
                // 首先检查取消令牌状态
                if child_cancel_play_token.is_cancelled() {
                    tracing::info!("{}: 收到取消信号，优雅退出播放任务", strategy_name);
                    *is_playing.write().await = false;
                    break;
                }
                
                // tracing::info!("{}: 播放k线，signal_count: {}, played_signal_count: {}", start_node.get_node_id().await, *signal_count.read().await, *played_signal_count.read().await);
                
                // 1. 判断是否为播放状态
                // 如果不是播放状态，则continue
                if !*is_playing.read().await {
                    tracing::info!("{}: 暂停播放, signal_count: {}, played_signal_count: {}", start_node.get_node_id().await, *signal_count.read().await, *played_signal_count.read().await);
                    
                    // 使用 tokio::select! 同时等待睡眠和取消信号
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                            continue;
                        }
                        _ = child_cancel_play_token.cancelled() => {
                            tracing::info!("{}: 在暂停状态收到取消信号，优雅退出播放任务", strategy_name);
                            *is_playing.write().await = false;
                            break;
                        }
                    }
                }

                // 2. 获取当前播放速度
                let play_speed = {
                    let speed = *initial_play_speed.read().await;
                    
                    // 确保 play_speed 在合理范围内（1-100）
                    if speed < 1 {
                        tracing::warn!("播放速度小于1，已调整为1");
                        1
                    } else if speed > 100 {
                        tracing::warn!("播放速度大于100，已调整为100");
                        100
                    } else {
                        speed
                    }
                };

                // 3. 获取信号计数和已发送的信号计数
                let signal_count = signal_count.read().await;
                let mut played_signal_count = played_signal_count.write().await;
                
                // 4. 如果已发送的信号计数小于等于信号计数，则发送信号
                if *played_signal_count <= *signal_count {
                    // 发送信号
                    start_node.send_fetch_data_signal(*played_signal_count).await;
                    // 更新已发送的信号计数
                    *played_signal_count += 1;
                }

                // 5. 如果已发送的信号计数大于信号计数，则停止播放
                if *played_signal_count > *signal_count {
                    // 发送k线播放完毕的信号
                    start_node.send_finish_signal().await;
                    tracing::info!("{}: k线播放完毕，正常退出播放任务", strategy_name);
                    *is_playing.write().await = false;
                    break;
                }

                // 根据播放速度计算延迟时间（毫秒）
                let delay_millis = 1000 / play_speed as u64;
                
                // 使用 tokio::select! 同时等待睡眠和取消信号
                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)) => {
                        // 正常继续下一次循环
                    }
                    _ = child_cancel_play_token.cancelled() => {
                        tracing::info!("{}: 在播放过程中收到取消信号，优雅退出播放任务", strategy_name);
                        *is_playing.write().await = false;
                        break;
                    }
                }
            }
            
            tracing::info!("{}: 播放任务已完全退出", strategy_name);
        });
    }

    // 暂停播放
    pub async fn pause(&mut self) {
        // 判断播放状态是否为true
        if !*self.is_playing.read().await {
            tracing::warn!("{}: 正在暂停，无需重复暂停", self.strategy_name.clone());
            return;
        }
        tracing::info!("{}: 请求暂停播放", self.strategy_name);
        self.cancel_play_token.cancel();
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
    }

    pub async fn stop(&mut self) {
        tracing::info!("{}: 停止播放", self.strategy_name.clone());
        self.cancel_play_token.cancel();
        // 重置信号计数
        *self.played_signal_count.write().await = 0;
        // 重置播放状态
        *self.is_playing.write().await = false;
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();

    }

    // 播放单根k线
    pub async fn play_one_kline(&self) {
        // 判断播放状态是否为true
        if *self.is_playing.read().await {
            tracing::warn!("{}: 正在播放，无法播放单根k线", self.strategy_name.clone());
            return;
        }

        // 检查是否已经播放完毕
        if *self.played_signal_count.read().await > *self.signal_count.read().await {
            tracing::warn!("{}: 已播放完毕，无法播放更多K线", self.strategy_name);
            return;
        }

        tracing::info!("{}: 开始播放单根k线", self.strategy_name.clone());
        // 获取开始节点的索引
        let start_node_index = self.node_indices.get("start_node").unwrap();
        let node = self.graph.node_weight(*start_node_index).unwrap().clone();
        let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();

        let signal_count = self.signal_count.read().await;
        let mut played_signal_count = self.played_signal_count.write().await;
        tracing::info!("{}: 播放单根k线，signal_count: {}, played_signal_count: {}", start_node.get_node_id().await, *signal_count, *played_signal_count);
        // 3. 如果已发送的信号计数小于等于信号计数，则发送信号
        if *played_signal_count <= *signal_count {
            // 发送信号
            start_node.send_fetch_data_signal(*played_signal_count).await;
            // 更新已发送的信号计数
            *played_signal_count += 1;

        }

        // 4. 如果已发送的信号计数大于信号计数，则停止播放
        if *played_signal_count > *signal_count {
            // 发送k线播放完毕的信号
            start_node.send_finish_signal().await;
        }
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