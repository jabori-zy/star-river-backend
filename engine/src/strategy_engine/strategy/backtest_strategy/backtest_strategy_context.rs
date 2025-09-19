pub mod playback_handler;

use super::super::StrategyCommandPublisher;
use crate::strategy_engine::node::node_state_machine::BacktestNodeRunState;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_state_machine::*;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use event_center::communication::strategy::{NodeCommandReceiver, BacktestStrategyResponse};
use event_center::communication::engine::cache_engine::{CacheEngineResponse, GetCacheMultiParams, GetCacheLengthMultiParams};
use event_center::communication::strategy::StrategyResponse;
use event_center::event::node_event::NodeEventTrait;
use event_center::singleton::EventCenterSingleton;
use event_center::event::strategy_event::backtest_strategy_event::BacktestStrategyEvent;
use event_center::event::Event;
use heartbeat::Heartbeat;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph};
use sea_orm::DatabaseConnection;
use snafu::IntoError;
use snafu::ResultExt;
use std::collections::HashMap;
use std::sync::Arc;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use star_river_core::cache::Key;
use star_river_core::custom_type::{NodeId, PlayIndex, StrategyId};
use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::position::virtual_position::VirtualPosition;
use event_center::event::node_event::backtest_node_event::futures_order_node_event::FuturesOrderNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::position_management_node_event::PositionManagementNodeEvent;
use event_center::event::node_event::backtest_node_event::IndicatorNodeEvent;
use event_center::event::node_event::backtest_node_event::{BacktestNodeEvent, CommonEvent};
use event_center::event::strategy_event::StrategyRunningLogEvent;
use event_center::communication::strategy::{NodeCommand, BacktestNodeCommand};
use event_center::communication::strategy::{GetCurrentTimeResponse, GetStrategyCacheKeysResponse, GetStartNodeConfigParams};
use star_river_core::strategy::strategy_inner_event::StrategyInnerEventPublisher;
use star_river_core::strategy::{BacktestStrategyConfig, StrategyConfig};
use star_river_core::strategy_stats::event::{StrategyStatsEvent, StrategyStatsEventReceiver};
use star_river_core::strategy_stats::StatsSnapshot;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use uuid::Uuid;
use virtual_trading::VirtualTradingSystem;
use chrono::{DateTime, Utc};

#[derive(Debug)]
// 回测策略上下文
pub struct BacktestStrategyContext {
    pub strategy_config: StrategyConfig,
    pub strategy_id: i32,
    pub strategy_name: String,                                  // 策略名称
    pub cache_lengths: HashMap<Key, u32>,                       // 缓存长度
    pub graph: Graph<Box<dyn BacktestNodeTrait>, (), Directed>, // 策略的拓扑图
    pub node_indices: HashMap<String, NodeIndex>,               // 节点索引
    pub cancel_task_token: CancellationToken,                   // 取消令牌
    pub state_machine: BacktestStrategyStateMachine,            // 策略状态机
    pub all_node_output_handles: Vec<NodeOutputHandle>,         // 接收策略内所有节点的消息
    pub database: DatabaseConnection,                           // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>,                       // 心跳
    // pub registered_tasks: Arc<RwLock<HashMap<String, Uuid>>>,   // 注册的任务 任务名称-> 任务id
    pub node_command_receiver: Arc<Mutex<Option<NodeCommandReceiver>>>, // 接收节点的命令
    pub strategy_command_publisher: StrategyCommandPublisher,           // 节点命令发送器
    pub total_signal_count: Arc<RwLock<i32>>,                           // 信号计数
    pub play_index: Arc<RwLock<i32>>,                                   // 播放索引
    pub is_playing: Arc<RwLock<bool>>,                                  // 是否正在播放
    pub initial_play_speed: Arc<RwLock<u32>>, // 初始播放速度 （从策略配置中加载）
    pub cancel_play_token: CancellationToken, // 取消播放令牌
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub strategy_inner_event_publisher: Option<StrategyInnerEventPublisher>, // 策略内部事件发布器
    pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>, // 策略统计模块
    pub strategy_stats_event_receiver: StrategyStatsEventReceiver, // 策略统计事件接收器
    pub(super) play_index_watch_tx: tokio::sync::watch::Sender<i32>, // 播放索引监听器
    pub(super) play_index_watch_rx: tokio::sync::watch::Receiver<i32>, // 播放索引监听器
    pub(super) leaf_node_ids: Vec<NodeId>,           // 叶子节点id
    pub(super) execute_over_node_ids: Arc<RwLock<Vec<NodeId>>>, // 执行完毕的节点id
    pub(super) execute_over_notify: Arc<Notify>,     // 已经更新播放索引的节点id通知
    pub(super) current_time: Arc<RwLock<DateTime<Utc>>>, // 当前时间
    pub(super) batch_id: Uuid,                       // 回测批次id
    pub(super) running_log: Arc<RwLock<Vec<StrategyRunningLogEvent>>>, // 运行日志
    pub(super) keys: Arc<RwLock<Vec<Key>>>,                            // 缓存键
    pub(super) min_interval_symbols: Vec<Key>,                        // 最小周期交易对
    
}

impl BacktestStrategyContext {
    pub fn new(
        strategy_config: StrategyConfig,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let strategy_id = strategy_config.id;
        let strategy_name = strategy_config.name.clone();
        let cancel_task_token = CancellationToken::new();
        let cancel_play_token = CancellationToken::new();

        let (play_index_watch_tx, play_index_watch_rx) =
            tokio::sync::watch::channel::<PlayIndex>(-1);
        let virtual_trading_system = Arc::new(Mutex::new(VirtualTradingSystem::new(
            play_index_watch_rx.clone(),
        )));

        let (strategy_stats_event_tx, strategy_stats_event_rx) =
            broadcast::channel::<StrategyStatsEvent>(100);
        let strategy_stats: Arc<RwLock<BacktestStrategyStats>> =
            Arc::new(RwLock::new(BacktestStrategyStats::new(
                strategy_id,
                virtual_trading_system.clone(),
                strategy_stats_event_tx,
                play_index_watch_rx.clone(),
            )));

        Self {
            strategy_config,
            strategy_id,
            strategy_name: strategy_name.clone(),
            keys: Arc::new(RwLock::new(vec![])),
            cache_lengths: HashMap::new(),
            graph: Graph::new(),
            node_indices: HashMap::new(),
            cancel_task_token,
            state_machine: BacktestStrategyStateMachine::new(
                strategy_id,
                strategy_name,
                BacktestStrategyRunState::Created,
            ),
            all_node_output_handles: vec![],
            database,
            heartbeat,
            // registered_tasks: Arc::new(RwLock::new(HashMap::new())),
            node_command_receiver: Arc::new(Mutex::new(None)),
            strategy_command_publisher: StrategyCommandPublisher::new(),
            total_signal_count: Arc::new(RwLock::new(0)),
            play_index: Arc::new(RwLock::new(-1)),
            is_playing: Arc::new(RwLock::new(false)),
            initial_play_speed: Arc::new(RwLock::new(0)),
            cancel_play_token,
            virtual_trading_system,
            strategy_inner_event_publisher: None,
            execute_over_notify: Arc::new(Notify::new()),
            strategy_stats,
            strategy_stats_event_receiver: strategy_stats_event_rx,
            play_index_watch_tx,
            play_index_watch_rx,
            leaf_node_ids: vec![],
            execute_over_node_ids: Arc::new(RwLock::new(vec![])),
            current_time: Arc::new(RwLock::new(Utc::now())),
            batch_id: Uuid::new_v4(),
            running_log: Arc::new(RwLock::new(vec![])),
            min_interval_symbols: vec![],
        }
    }

    pub fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }

    pub async fn get_keys(&self) -> Vec<Key> {
        self.keys.read().await.clone()
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

    pub fn set_node_command_receiver(&mut self, node_command_receiver: NodeCommandReceiver) {
        self.node_command_receiver = Arc::new(Mutex::new(Some(node_command_receiver)));
    }

    pub fn set_strategy_inner_event_publisher(
        &mut self,
        strategy_inner_event_publisher: StrategyInnerEventPublisher,
    ) {
        self.strategy_inner_event_publisher = Some(strategy_inner_event_publisher);
    }

    pub async fn get_current_time(&self) -> DateTime<Utc> {
        self.current_time.read().await.clone()
    }

    pub async fn set_current_time(&mut self, current_time: DateTime<Utc>) {
        *self.current_time.write().await = current_time;
    }

    pub async fn get_running_log(&self) -> Vec<StrategyRunningLogEvent> {
        self.running_log.read().await.clone()
    }

    pub async fn add_running_log(&mut self, running_log: StrategyRunningLogEvent) {
        self.running_log.write().await.push(running_log);
    }

    pub async fn reset_running_log(&mut self) {
        self.running_log.write().await.clear();
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<Key>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols(&self) -> Vec<Key> {
        self.min_interval_symbols.clone()
    }

    pub fn set_all_node_output_handles(&mut self, all_node_output_handles: Vec<NodeOutputHandle>) {
        self.all_node_output_handles = all_node_output_handles;
    }

    pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_ids = leaf_node_ids;
    }

    pub fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle> {
        self.all_node_output_handles.clone()
    }

    pub fn get_cancel_task_token(&self) -> CancellationToken {
        self.cancel_task_token.clone()
    }

    pub fn get_node_command_receiver(&self) -> Arc<Mutex<Option<NodeCommandReceiver>>> {
        self.node_command_receiver.clone()
    }

    pub async fn handle_node_command(&mut self, command: NodeCommand) -> Result<(), String> {
        match command {
            NodeCommand::BacktestNode(BacktestNodeCommand::GetStrategyKeys(
                get_strategy_cache_keys_command,
            )) => {
                let keys = self.get_keys().await;
                let get_strategy_cache_keys_response = GetStrategyCacheKeysResponse::success(keys);
                get_strategy_cache_keys_command
                    .responder
                    .send(get_strategy_cache_keys_response.into())
                    .unwrap();
            }
            NodeCommand::BacktestNode(BacktestNodeCommand::GetCurrentTime(
                get_current_time_command,
            )) => {
                let current_time = self.get_current_time().await;
                let get_current_time_response = GetCurrentTimeResponse::success(current_time);
                get_current_time_command
                    .responder
                    .send(get_current_time_response.into())
                    .unwrap();
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
    pub async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // 执行完毕
        if let BacktestNodeEvent::Common(signal_event) = &node_event {
            match signal_event {
                // 执行结束
                CommonEvent::ExecuteOver(execute_over_event) => {
                    // tracing::debug!("{}: 收到执行完毕事件: {:?}", self.strategy_name.clone(), execute_over_event);
                    // tracing::debug!("leaf_node_ids: {:#?}", self.leaf_node_ids);
                    let mut execute_over_node_ids = self.execute_over_node_ids.write().await;
                    if !execute_over_node_ids.contains(&execute_over_event.from_node_id()) {
                        execute_over_node_ids.push(execute_over_event.from_node_id().clone());
                    }

                    // 如果所有叶子节点都执行完毕，则通知等待的线程
                    if execute_over_node_ids.len() == self.leaf_node_ids.len() {
                        tracing::debug!(
                            "{}: 所有叶子节点执行完毕, 通知等待的线程。叶子节点id: {:?}",
                            self.strategy_name.clone(),
                            execute_over_node_ids
                        );
                        self.execute_over_notify.notify_waiters();
                        // 通知完成后，清空execute_over_node_ids
                        execute_over_node_ids.clear();
                    }
                }
                CommonEvent::RunningLog(running_log_event) => {
                    self.add_running_log(running_log_event.clone()).await;
                    let backtest_strategy_event: BacktestStrategyEvent = running_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event)
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }

        if let BacktestNodeEvent::KlineNode(kline_node_event) = &node_event {
            match kline_node_event {
                KlineNodeEvent::KlineUpdate(kline_update_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::KlineUpdate(kline_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                KlineNodeEvent::StateLog(log_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::NodeStateLog(log_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                KlineNodeEvent::TimeUpdate(time_update_event) => {
                    // 更新策略的全局时间
                    self.set_current_time(time_update_event.current_time).await;
                }
                _ => {}
            }
        }

        if let BacktestNodeEvent::IndicatorNode(indicator_node_event) = &node_event {
            match indicator_node_event {
                IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::IndicatorUpdate(indicator_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }

        // 期货订单节点事件
        if let BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) = &node_event {
            match futures_order_node_event {
                FuturesOrderNodeEvent::FuturesOrderFilled(futures_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderFilled(
                        futures_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCreated(futures_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCreated(
                        futures_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCanceled(futures_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCanceled(
                        futures_order_canceled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCreated(take_profit_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCreated(
                        take_profit_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCreated(stop_loss_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCreated(
                        stop_loss_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderFilled(take_profit_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderFilled(
                        take_profit_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderFilled(stop_loss_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderFilled(
                        stop_loss_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCanceled(
                    take_profit_order_canceled_event,
                ) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCanceled(
                        take_profit_order_canceled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCanceled(stop_loss_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCanceled(
                        stop_loss_order_canceled_event.clone(),
                    );
                    //  let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TransactionCreated(transaction_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TransactionCreated(
                        transaction_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }

        if let BacktestNodeEvent::PositionManagementNode(position_management_node_event) =
            &node_event
        {
            match position_management_node_event {
                PositionManagementNodeEvent::PositionCreated(position_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionCreated(position_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                PositionManagementNodeEvent::PositionUpdated(position_updated_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionUpdated(position_updated_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                PositionManagementNodeEvent::PositionClosed(position_closed_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }
    }

    pub async fn handle_strategy_stats_event(
        &mut self,
        event: StrategyStatsEvent,
    ) -> Result<(), String> {
        match event {
            StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event) => {
                // tracing::debug!("{}: 收到策略统计更新事件: {:?}", self.strategy_name, strategy_stats_updated_event);
                let backtest_strategy_event =
                    BacktestStrategyEvent::StrategyStatsUpdated(strategy_stats_updated_event);
                // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                EventCenterSingleton::publish(backtest_strategy_event.into())
                    .await
                    .unwrap();
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

    pub async fn get_node_list(&self) -> Vec<Vec<String>> {
        let nodes = self.topological_sort();
        let mut node_list = Vec::new();
        for node in &nodes {
            let node_name = node.get_node_name().await;
            let node_id = node.get_node_id().await;
            node_list.push(vec![node_id, node_name]);
        }
        node_list
    }

    async fn get_strategy_data(
        strategy_id: StrategyId,
        strategy_name: String,
        cache_keys: Arc<RwLock<Vec<Key>>>,
    ) {
        let cache_keys_clone = cache_keys.read().await.clone();
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheMultiParams::new(
            strategy_id,
            cache_keys_clone,
            None,
            None,
            strategy_name,
            resp_tx,
        );

        let _ = EventCenterSingleton::send_command(params.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
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
                tracing::info!(
                    "所有节点已停止，共耗时{}ms",
                    start_time.elapsed().as_millis()
                );
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

    // 初始化所有节点的方法，不持有外部锁
    pub async fn init_node(
        context: Arc<RwLock<Self>>,
    ) -> Result<(), BacktestStrategyError> {
        // 短暂持有锁获取节点列表
        let nodes = {
            let context_guard = context.read().await;
            context_guard.topological_sort()
        }; // 锁立即释放

        // 逐个初始化节点，不持有锁
        for node in nodes {
            let mut node_clone = node.clone();

            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> =
                tokio::spawn(async move {
                    node_clone.init().await.context(NodeInitSnafu {})?;
                    Ok(())
                });

            let node_name = node.get_node_name().await;
            let node_id = node.get_node_id().await;
            let node_type = node.get_node_type().await;

            // 等待节点初始化完成（这里没有持有任何锁）
            match tokio::time::timeout(Duration::from_secs(30), node_handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            task_name: "INIT_NODE".to_string(),
                            node_name,
                            node_id,
                            node_type: node_type.to_string(),
                        }
                        .into_error(e));
                    }

                    if let Ok(Err(e)) = result {
                        return Err(e);
                    }
                }
                Err(e) => {
                    return Err(NodeInitTimeoutSnafu {
                        node_id,
                        node_name,
                        node_type: node_type.to_string(),
                    }
                    .into_error(e));
                }
            }

            // 等待节点进入Ready状态（这里也没有持有锁）
            let mut retry_count = 0;
            let max_retries = 10;

            while retry_count < max_retries {
                let run_state = node.get_run_state().await;
                if run_state == BacktestNodeRunState::Ready {
                    tracing::debug!("节点 {} 已进入Ready状态", node_id);
                    // 节点初始化间隔
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    break;
                }
                retry_count += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            if retry_count >= max_retries {
                return Err(NodeStateNotReadySnafu {
                    node_id: node_id,
                    node_name: node_name,
                    node_type: node_type.to_string(),
                }
                .fail()?);
            }
        }

        Ok(())
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
                tokio::time::sleep(Duration::from_millis(10)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Err(format!("节点 {} 未能进入Stopped状态", node_id))
    }

    // #[instrument(skip(self))]
    // // 获取所有k线缓存中的最小长度
    // pub async fn get_cache_length(&self) -> Result<HashMap<Key, u32>, String> {
    //     // 过滤出k线缓存key
    //     let kline_cache_keys = self
    //         .keys
    //         .read()
    //         .await
    //         .iter()
    //         .filter(|cache_key| matches!(cache_key, Key::Kline(_)))
    //         .map(|cache_key| cache_key.clone())
    //         .collect();
    //     let (resp_tx, resp_rx) = oneshot::channel();
    //     let get_cache_length_params = GetCacheLengthMultiParams::new(
    //         self.strategy_id,
    //         kline_cache_keys,
    //         self.strategy_name.clone(),
    //         resp_tx,
    //     );
    //     // 向缓存引擎发送命令
    //     // self.command_publisher.send(cache_engine_command.into()).await.unwrap();
    //     EventCenterSingleton::send_command(get_cache_length_params.into())
    //         .await
    //         .unwrap();
    //     let response = resp_rx.await.unwrap();
    //     if response.success() {
    //         let cache_engine_response = CacheEngineResponse::try_from(response);
    //         if let Ok(cache_engine_response) = cache_engine_response {
    //             match cache_engine_response {
    //                 CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response) => {
    //                     tracing::info!(cache_lengths = ?get_cache_length_multi_response.cache_length, "Get cache length successfully!");
    //                     Ok(get_cache_length_multi_response.cache_length)
    //                 }
    //                 _ => Err("get cache length multi failed".to_string()),
    //             }
    //         } else {
    //             Err("try from response failed".to_string())
    //         }
    //     } else {
    //         Err("get cache length multi failed".to_string())
    //     }
    // }

    // 初始化信号计数
    #[instrument(skip(self))]
    pub async fn get_signal_count(&mut self) -> Result<i32, String> {
        // // 初始化信号计数
        // let min_cache_length = self.cache_lengths.values().min().cloned().unwrap_or(0);
        // Ok(min_cache_length as i32)
        let min_interval_keys = self.get_min_interval_symbols();

        // 1. 从缓存引擎获取每一个symbol的缓存长度
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_length_params = GetCacheLengthMultiParams::new(
            self.strategy_id,
            min_interval_keys.clone(),
            self.strategy_name.clone(),
            resp_tx,
        );
        EventCenterSingleton::send_command(get_cache_length_params.into())
            .await
            .unwrap();
        let response = resp_rx.await.unwrap();
        if response.success() {
            let cache_engine_response = CacheEngineResponse::try_from(response);
            if let Ok(cache_engine_response) = cache_engine_response {
                match cache_engine_response {
                    CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response) => {
                        let symbol_cache_lengths = get_cache_length_multi_response.cache_length;

                        // 2. 判断每一个symbol的缓存长度是否相同
                        if symbol_cache_lengths.is_empty() {
                            return Err("no symbol cache lengths found".to_string());
                        }

                        // 获取第一个symbol的缓存长度作为基准
                        let min_cache_length = symbol_cache_lengths.values().min().cloned().unwrap_or(0);

                        // 检查所有symbol的缓存长度是否都相同
                        for (key, cache_length) in symbol_cache_lengths.iter() {
                            if *cache_length != min_cache_length {
                                return Err(format!("symbol {} cache length {} is not same as min cache length {}", key.get_symbol(), cache_length, min_cache_length));
                            }
                        }

                        tracing::debug!("symbol_cache_lengths: {:#?}, min_cache_length: {}", symbol_cache_lengths, min_cache_length);

                        return Ok(min_cache_length as i32);
                    }
                    _ => {
                        return Err("get cache length multi failed".to_string());
                    }
                }
            } else {
                return Err("try from response failed".to_string());
            }
        } else {
            return Err("get cache length multi failed".to_string());
        }
    }

    // 获取start节点配置
    pub async fn get_start_node_config(&self) -> Result<BacktestStrategyConfig, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_start_node_config_params =
            GetStartNodeConfigParams::new("start_node".to_string(), resp_tx);
        self.strategy_command_publisher
            .send(get_start_node_config_params.into())
            .await
            .unwrap();
        // EventCenterSingleton::send_command(get_start_node_config_command).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let StrategyResponse::BacktestStrategy(
                BacktestStrategyResponse::GetStartNodeConfig(get_start_node_config_response),
            ) = response
            {
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
        let current_positions = virtual_trading_system.get_current_positions_ref();
        current_positions.clone()
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let history_positions = virtual_trading_system.get_history_positions();
        history_positions
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let transactions = virtual_trading_system.get_transactions();
        transactions
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        let strategy_stats = self.strategy_stats.read().await;
        strategy_stats.get_stats_history(play_index).await
    }

    pub async fn virtual_trading_system_reset(&self) {
        let mut virtual_trading_system = self.virtual_trading_system.lock().await;
        virtual_trading_system.reset();
    }

    pub async fn strategy_stats_reset(&self) {
        let mut strategy_stats = self.strategy_stats.write().await;
        strategy_stats.clear_asset_snapshots().await;
    }

    pub async fn update_strategy_status(
        &mut self,
        status: String,
    ) -> Result<(), BacktestStrategyError> {
        let strategy_id = self.strategy_id;
        StrategyConfigMutation::update_strategy_status(&self.database, strategy_id, status)
            .await
            .context(UpdateStrategyStatusFailedSnafu {
                strategy_id,
                strategy_name: self.strategy_name.clone(),
            })?;
        Ok(())
    }
}
