use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use types::custom_type::{NodeId, StrategyId};
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_state_machine::*;
use types::strategy::node_event::{NodeEvent, SignalEvent};
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock,Notify};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_state_machine::BacktestNodeRunState;
use types::cache::CacheKey;
use uuid::Uuid;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheMultiParams};
use event_center::response::cache_engine_response::CacheEngineResponse;
use utils::get_utc8_timestamp_millis;
use event_center::command::Command;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use tokio::sync::oneshot;
use types::strategy::BacktestStrategyConfig;
use types::strategy::node_command::{NodeCommandReceiver, NodeCommand};
use types::strategy::node_response::{GetStrategyCacheKeysResponse};
use tracing::instrument;
use event_center::command::cache_engine_command::GetCacheLengthMultiParams;
use virtual_trading::VirtualTradingSystem;
use types::strategy::node_response::NodeResponse;
use types::strategy::strategy_inner_event::StrategyInnerEventPublisher;


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
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub registered_tasks: Arc<RwLock<HashMap<String, Uuid>>>, // 注册的任务 任务名称-> 任务id
    pub node_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 节点命令接收器
    pub signal_count: Arc<RwLock<u32>>, // 信号计数
    pub played_signal_index: Arc<RwLock<u32>>, // 已发送的信号计数
    pub is_playing: Arc<RwLock<bool>>, // 是否正在播放
    pub initial_play_speed: Arc<RwLock<u32>>, // 初始播放速度 （从策略配置中加载）
    pub cancel_play_token: CancellationToken, // 取消播放令牌
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub strategy_inner_event_publisher: StrategyInnerEventPublisher, // 策略内部事件发布器
    pub updated_play_index_node_ids: Arc<RwLock<Vec<NodeId>>>, // 已经更新播放索引的节点id
    pub updated_play_index_notify: Arc<Notify>, // 已经更新播放索引的节点id通知
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

    pub fn get_event_receivers(&self) -> &Vec<broadcast::Receiver<Event>> {
        &self.event_receivers
    }

    pub fn get_command_receiver(&self) -> Arc<Mutex<NodeCommandReceiver>> {
        self.node_command_receiver.clone()
    }

    pub async fn handle_node_command(&mut self, command: NodeCommand) -> Result<(), String> {
        match command {
            NodeCommand::GetStrategyCacheKeys(get_strategy_cache_keys_command) => {
                let cache_keys = self.get_cache_keys().await;
                let get_strategy_cache_keys_response = NodeResponse::GetStrategyCacheKeys(GetStrategyCacheKeysResponse {
                    code: 0,
                    message: "success".to_string(),
                    cache_keys: cache_keys,
                    response_timestamp: get_utc8_timestamp_millis(),
                });
                get_strategy_cache_keys_command.responder.send(get_strategy_cache_keys_response).unwrap();
            }
            _ => {}
        }
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

    // 所有节点发送的事件都会汇集到这里
    pub async fn handle_node_events(&self, node_event: NodeEvent) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.strategy_name, node_event);
        match node_event {
            NodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::PlayIndexUpdated(play_index_update_event) => {
                        tracing::debug!("{}: play index 已更新: {:?}", play_index_update_event.from_node_id, play_index_update_event.node_play_index);
                        // 如果节点id不在updated_play_index_node_ids中，则添加到updated_play_index_node_ids中
                        let mut updated_play_index_node_ids = self.updated_play_index_node_ids.write().await;
                        if !updated_play_index_node_ids.contains(&play_index_update_event.from_node_id) {
                                    updated_play_index_node_ids.push(play_index_update_event.from_node_id.clone());
                        }
                        
                        // 如果所有节点索引更新完毕，则通知等待的线程
                        if updated_play_index_node_ids.len() == self.graph.node_count() {
                            tracing::debug!("{}: 所有节点索引更新完毕, 通知等待的线程", self.strategy_name.clone());
                            self.updated_play_index_notify.notify_waiters();
                            // 通知完成后，清空updated_play_index_node_ids
                            updated_play_index_node_ids.clear();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
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





}