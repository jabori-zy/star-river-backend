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
use types::strategy::node_event::{BacktestNodeEvent, SignalEvent};
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock,Notify};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_state_machine::BacktestNodeRunState;
use types::cache::Key;
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
use event_center::strategy_event::backtest_strategy_event::BacktestStrategyEvent;
use types::strategy::node_event::IndicatorNodeEvent;
use super::super::StrategyCommandPublisher;
use event_center::command::backtest_strategy_command::{StrategyCommand, GetStartNodeConfigParams};
use event_center::response::backtest_strategy_response::StrategyResponse;
use types::strategy::node_event::backtest_node_event::futures_order_node_event::FuturesOrderNodeEvent;
use types::order::virtual_order::VirtualOrder;
use types::strategy::node_event::backtest_node_event::position_management_node_event::PositionManagementNodeEvent;
use types::position::virtual_position::VirtualPosition;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use types::strategy_stats::event::StrategyStatsEventReceiver;


#[derive(Debug)]
// 实盘策略上下文
pub struct BacktestStrategyContext {
    pub strategy_id: i32,
    pub strategy_name: String, // 策略名称
    pub cache_keys: Arc<RwLock<Vec<Key>>>, // 缓存键
    pub cache_lengths: HashMap<Key, u32>, // 缓存长度
    pub graph: Graph<Box<dyn BacktestNodeTrait>, (),  Directed>, // 策略的拓扑图
    pub node_indices: HashMap<String, NodeIndex>, // 节点索引
    pub event_publisher: EventPublisher, // 外部事件发布器
    pub event_receivers: Vec<EventReceiver>, // 外部事件接收器
    pub command_publisher: CommandPublisher, // 外部命令发布器
    pub command_receiver: Arc<Mutex<CommandReceiver>>, // 外部命令接收器
    pub cancel_token: CancellationToken, // 取消令牌
    pub state_machine: BacktestStrategyStateMachine, // 策略状态机
    pub all_node_output_handles: Vec<NodeOutputHandle>, // 接收策略内所有节点的消息
    pub database: DatabaseConnection, // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    pub registered_tasks: Arc<RwLock<HashMap<String, Uuid>>>, // 注册的任务 任务名称-> 任务id
    pub node_command_receiver: Arc<Mutex<NodeCommandReceiver>>, // 接收节点的命令
    pub strategy_command_publisher: StrategyCommandPublisher, // 节点命令发送器
    pub total_signal_count: Arc<RwLock<i32>>, // 信号计数
    pub play_index: Arc<RwLock<i32>>, // 播放索引
    pub is_playing: Arc<RwLock<bool>>, // 是否正在播放
    pub initial_play_speed: Arc<RwLock<u32>>, // 初始播放速度 （从策略配置中加载）
    pub cancel_play_token: CancellationToken, // 取消播放令牌
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub strategy_inner_event_publisher: StrategyInnerEventPublisher, // 策略内部事件发布器
    pub updated_play_index_node_ids: Arc<RwLock<Vec<NodeId>>>, // 已经更新播放索引的节点id
    pub updated_play_index_notify: Arc<Notify>, // 已经更新播放索引的节点id通知
    pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>,   // 策略统计模块
    pub strategy_stats_event_receiver: StrategyStatsEventReceiver, // 策略统计事件接收器
    pub play_index_watch_tx: tokio::sync::watch::Sender<i32>, // 播放索引监听器
}


impl BacktestStrategyContext {
    pub fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }

    pub async fn get_cache_keys(&self) -> Vec<Key> {
        self.cache_keys.read().await.clone()
    }

    // 获取节点
    pub fn get_node(&self, node_id: &str) -> Option<Box<dyn BacktestNodeTrait>> {
        let node_index = self.node_indices.get(node_id);
        if let Some(node_index) = node_index {
            let node = self.graph.node_weight(*node_index);
            if let Some(node) = node {
                Some(node.clone())
            } else {
                None
            }
        } else {
            None
        }
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
    pub async fn handle_node_event(&self, node_event: BacktestNodeEvent) -> Result<(), String> {
        // 播放索引更新事件
        // if let BacktestNodeEvent::Signal(signal_event) = &node_event {
        //     match signal_event {
        //         SignalEvent::PlayIndexUpdated(play_index_update_event) => {
        //             // tracing::debug!("{}: play index 已更新: {:?}", play_index_update_event.from_node_id, play_index_update_event.play_index);
        //             // 如果节点id不在updated_play_index_node_ids中，则添加到updated_play_index_node_ids中
        //             let mut updated_play_index_node_ids = self.updated_play_index_node_ids.write().await;
        //             if !updated_play_index_node_ids.contains(&play_index_update_event.from_node_id) {
        //                         updated_play_index_node_ids.push(play_index_update_event.from_node_id.clone());
        //             }
                    
        //             // 如果所有节点索引更新完毕，则通知等待的线程
        //             if updated_play_index_node_ids.len() == self.graph.node_count() {
        //                 // tracing::debug!("{}: 所有节点索引更新完毕, 通知等待的线程", self.strategy_name.clone());
        //                 self.updated_play_index_notify.notify_waiters();
        //                 // 通知完成后，清空updated_play_index_node_ids
        //                 updated_play_index_node_ids.clear();
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        if let BacktestNodeEvent::KlineNode(kline_node_event)  = &node_event {
            match kline_node_event {
                KlineNodeEvent::KlineUpdate(kline_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::KlineUpdate(kline_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
            }
        }

        if let BacktestNodeEvent::IndicatorNode(indicator_node_event) = &node_event {
            match indicator_node_event {
                IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::IndicatorUpdate(indicator_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
                _ => {}
            }
        }

        // 期货订单节点事件
        if let BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) = &node_event {
            match futures_order_node_event {
                FuturesOrderNodeEvent::FuturesOrderFilled(futures_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderFilled(futures_order_filled_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
                FuturesOrderNodeEvent::FuturesOrderCreated(futures_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCreated(futures_order_created_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
                FuturesOrderNodeEvent::FuturesOrderCanceled(futures_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCanceled(futures_order_canceled_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
            }
        }

        if let BacktestNodeEvent::PositionManagementNode(position_management_node_event) = &node_event {
            match position_management_node_event {
                PositionManagementNodeEvent::PositionCreated(position_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionCreated(position_created_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
                PositionManagementNodeEvent::PositionUpdated(position_updated_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionUpdated(position_updated_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
                PositionManagementNodeEvent::PositionClosed(position_closed_event) => { 
                    let backtest_strategy_event = BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                }
            }
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


    async fn get_strategy_data(
        strategy_id: StrategyId,
        strategy_name: String,
        cache_keys: Arc<RwLock<Vec<Key>>>,
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
        let mut node_clone = node.clone();
        let node_name = node_clone.get_node_name().await;
        let node_id = node_clone.get_node_id().await;
        
        let node_handle = tokio::spawn(async move {
            if let Err(e) = node_clone.stop().await {
                tracing::error!(node_name = %node_name, node_id = %node_id, error = %e, "节点停止失败。");
                return Err(format!("节点停止失败。"));
            }
            Ok(())
        });

        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;
        
        
        // 等待节点停止完成
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
        
        // 等待节点进入Stopped状态
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
    pub async fn get_cache_length(&self) -> Result<HashMap<Key, u32>, String> {
        
        // 过滤出k线缓存key
        let kline_cache_keys = self.cache_keys
            .read()
            .await
            .iter()
            .filter(|cache_key| matches!(cache_key, Key::Kline(_)))
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
    pub async fn get_signal_count(&mut self) -> Result<i32, String> {
        // 初始化信号计数
        let min_cache_length = self.cache_lengths.values().min().cloned().unwrap_or(0);
        Ok(min_cache_length as i32)
    }

    // 获取start节点配置
    pub async fn get_start_node_config(&self) -> Result<BacktestStrategyConfig, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_start_node_config_command = StrategyCommand::GetStartNodeConfig(GetStartNodeConfigParams {
            node_id: "start_node".to_string(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        });
        self.strategy_command_publisher.send(get_start_node_config_command).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            if let StrategyResponse::GetStartNodeConfig(get_start_node_config_response) = response {
                Ok(get_start_node_config_response.backtest_strategy_config)
            } else {
                Err("get start node config failed".to_string())
            }
            
        } else {
            Err("get start node config failed".to_string())
        }

    }

    pub async fn get_play_index(&self) -> i32 {
        let play_index = self.play_index.read().await;
        *play_index
    }

    
    // 获取所有的virtual order
    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let virtual_orders = virtual_trading_system.get_virtual_orders();
        virtual_orders
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let current_positions = virtual_trading_system.get_current_positions();
        current_positions
    }
    
    pub async fn virtual_trading_system_reset(&self) {
        let mut virtual_trading_system = self.virtual_trading_system.lock().await;
        virtual_trading_system.reset();
    }






}