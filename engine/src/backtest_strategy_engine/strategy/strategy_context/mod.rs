mod command_handler;
mod data_handler;
mod event_handler;
mod node_lifecycle;
mod node_operation;
mod playback_handler;
mod strategy_operation;

use super::BacktestNodeTrait;
use super::node_state_machine::BacktestNodeRunState;
use super::strategy_state_machine::{BacktestStrategyRunState,BacktestStrategyStateMachine};
use chrono::{DateTime, Utc};
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use event_center::communication::Command;
use event_center::communication::backtest_strategy::*;
use event_center::event::Event;
use event_center::event::node_event::NodeEventTrait;
use event_center::event::node_event::backtest_node_event::{BacktestNodeEvent,CommonEvent,
    futures_order_node_event::FuturesOrderNodeEvent,
    indicator_node_event::IndicatorNodeEvent,
    kline_node_event::KlineNodeEvent,
    position_management_node_event::PositionManagementNodeEvent,
};
use event_center::event::strategy_event::{StrategyRunningLogEvent,
    backtest_strategy_event::{PlayFinishedEvent,BacktestStrategyEvent}
};

use event_center::singleton::EventCenterSingleton;
use heartbeat::Heartbeat;
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph};
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::key::{Key, KeyTrait,
    key::{IndicatorKey, KlineKey},
};
use star_river_core::market::{Kline, QuantData};
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::position::virtual_position::VirtualPosition;
use star_river_core::strategy::custom_variable::CustomVariable;
use star_river_core::strategy::node_benchmark::CompletedCycle;
use star_river_core::strategy::sys_varibale::{SysVariable, SysVariableType};
use star_river_core::strategy::{BacktestStrategyConfig, StrategyConfig};
use star_river_core::strategy_stats::{StatsSnapshot,
    event::{StrategyStatsEvent, StrategyStatsEventReceiver}
};
use star_river_core::system::DateTimeUtc;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{broadcast, mpsc, Mutex, Notify, RwLock};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use virtual_trading::VirtualTradingSystem;
use star_river_core::strategy::strategy_benchmark::{StrategyBenchmark, StrategyCycleTracker};

#[derive(Debug)]
// 回测策略上下文
pub struct BacktestStrategyContext {
    pub strategy_config: StrategyConfig,
    pub strategy_id: i32,
    pub strategy_name: String,                                  // 策略名称
    pub graph: Graph<Box<dyn BacktestNodeTrait>, (), Directed>, // 策略的拓扑图
    pub node_indices: HashMap<String, NodeIndex>,               // 节点索引
    pub cancel_task_token: CancellationToken,                   // 取消令牌
    pub state_machine: BacktestStrategyStateMachine,            // 策略状态机
    // pub all_node_output_handles: Vec<NodeOutputHandle>,         // 接收策略内所有节点的消息
    pub database: DatabaseConnection,                           // 数据库连接
    pub heartbeat: Arc<Mutex<Heartbeat>>,                       // 心跳
    pub strategy_command_sender: StrategyCommandSender,
    pub strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>, // 接收节点的命令
    pub node_command_sender: HashMap<NodeId, NodeCommandSender>,        // 节点命令发送器
    pub total_signal_count: Arc<RwLock<i32>>,                           // 信号计数
    pub play_index: Arc<RwLock<i32>>,                                   // 播放索引
    pub is_playing: Arc<RwLock<bool>>,                                  // 是否正在播放
    pub initial_play_speed: Arc<RwLock<u32>>,                           // 初始播放速度 （从策略配置中加载）
    pub cancel_play_token: CancellationToken,                           // 取消播放令牌
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,       // 虚拟交易系统
    pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>,             // 策略统计模块
    pub strategy_stats_event_receiver: StrategyStatsEventReceiver,      // 策略统计事件接收器
    pub(super) play_index_watch_tx: tokio::sync::watch::Sender<i32>,    // 播放索引监听器
    pub(super) play_index_watch_rx: tokio::sync::watch::Receiver<i32>,  // 播放索引监听器
    pub(super) leaf_node_ids: Vec<NodeId>,                              // 叶子节点id
    pub(super) execute_over_node_ids: Arc<RwLock<Vec<NodeId>>>,         // 执行完毕的节点id
    pub(super) execute_over_notify: Arc<Notify>,                        // 已经更新播放索引的节点id通知
    pub(super) current_time: Arc<RwLock<DateTime<Utc>>>,                // 当前时间
    pub(super) batch_id: Uuid,                                          // 回测批次id
    pub(super) running_log: Arc<RwLock<Vec<StrategyRunningLogEvent>>>,  // 运行日志
    pub(super) keys: Arc<RwLock<HashMap<Key, NodeId>>>,                 // 缓存键 -> 其所属节点id
    pub(super) min_interval_symbols: Vec<KlineKey>,                     // 最小周期交易对
    pub(super) kline_data: Arc<RwLock<HashMap<KlineKey, Vec<Kline>>>>,  // 所有k线数据
    pub(super) indicator_data: Arc<RwLock<HashMap<IndicatorKey, Vec<Indicator>>>>, // 所有指标数据
    pub(super) custom_variable: Arc<RwLock<HashMap<String, CustomVariable>>>, // var_name -> CustomVariable
    pub(super) sys_variable: Arc<RwLock<HashMap<SysVariableType, SysVariable>>>, // var_name -> SysVariable
    benchmark: Arc<RwLock<StrategyBenchmark>>,
    cycle_tracker: Arc<RwLock<Option<StrategyCycleTracker>>>,
}

impl BacktestStrategyContext {
    pub fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let strategy_id = strategy_config.id;
        let strategy_name = strategy_config.name.clone();
        let cancel_task_token = CancellationToken::new();
        let cancel_play_token = CancellationToken::new();

        // strategy command sender and receiver
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<BacktestStrategyCommand>(100);

        let (play_index_watch_tx, play_index_watch_rx) = tokio::sync::watch::channel::<PlayIndex>(-1);

        // new virtual trading system
        let virtual_trading_system = Arc::new(Mutex::new(VirtualTradingSystem::new(
            play_index_watch_rx.clone(),
            strategy_command_tx.clone(),
        )));

        let (strategy_stats_event_tx, strategy_stats_event_rx) = broadcast::channel::<StrategyStatsEvent>(100);
        let strategy_stats: Arc<RwLock<BacktestStrategyStats>> = Arc::new(RwLock::new(BacktestStrategyStats::new(
            strategy_id,
            virtual_trading_system.clone(),
            strategy_stats_event_tx,
            play_index_watch_rx.clone(),
        )));

        let benchmark = Arc::new(RwLock::new(StrategyBenchmark::new(strategy_id, strategy_name.clone())));

        Self {
            strategy_config,
            strategy_id,
            strategy_name: strategy_name.clone(),
            keys: Arc::new(RwLock::new(HashMap::new())),
            graph: Graph::new(),
            node_indices: HashMap::new(),
            cancel_task_token,
            state_machine: BacktestStrategyStateMachine::new(strategy_id, strategy_name, BacktestStrategyRunState::Created),
            // all_node_output_handles: vec![],
            database,
            heartbeat,
            strategy_command_sender: strategy_command_tx,
            strategy_command_receiver: Arc::new(Mutex::new(strategy_command_rx)),
            node_command_sender: HashMap::new(),
            total_signal_count: Arc::new(RwLock::new(0)),
            play_index: Arc::new(RwLock::new(-1)),
            is_playing: Arc::new(RwLock::new(false)),
            initial_play_speed: Arc::new(RwLock::new(0)),
            cancel_play_token,
            virtual_trading_system,
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
            kline_data: Arc::new(RwLock::new(HashMap::new())),
            indicator_data: Arc::new(RwLock::new(HashMap::new())),
            custom_variable: Arc::new(RwLock::new(HashMap::new())),
            sys_variable: Arc::new(RwLock::new(HashMap::new())),
            benchmark,
            cycle_tracker: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_strategy_name(&self) -> String {
        self.strategy_name.clone()
    }

    pub async fn get_keys(&self) -> HashMap<Key, NodeId> {
        self.keys.read().await.clone()
    }

    // 获取节点
    pub fn get_node(&self, node_id: &str) -> Option<Box<dyn BacktestNodeTrait>> {
        let node_index = self.node_indices.get(node_id);
        if let Some(node_index) = node_index {
            let node = self.graph.node_weight(*node_index);
            if let Some(node) = node { Some(node.clone()) } else { None }
        } else {
            None
        }
    }

    pub fn set_state_machine(&mut self, state_machine: BacktestStrategyStateMachine) {
        self.state_machine = state_machine;
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

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols(&self) -> Vec<KlineKey> {
        self.min_interval_symbols.clone()
    }

    // pub fn set_all_node_output_handles(&mut self, all_node_output_handles: Vec<NodeOutputHandle>) {
    //     self.all_node_output_handles = all_node_output_handles;
    // }

    pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_ids = leaf_node_ids;
    }

    // pub fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle> {
    //     self.all_node_output_handles.clone()
    // }

    pub fn get_cancel_task_token(&self) -> CancellationToken {
        self.cancel_task_token.clone()
    }

    pub fn get_strategy_command_receiver(&self) -> Arc<Mutex<StrategyCommandReceiver>> {
        self.strategy_command_receiver.clone()
    }

    // 添加节点命令发送器
    pub async fn add_node_command_sender(&mut self, node_id: NodeId, sender: NodeCommandSender) {
        self.node_command_sender.insert(node_id, sender);
    }

    pub async fn send_node_command(&self, node_command: BacktestNodeCommand) {
        self.node_command_sender
            .get(&node_command.node_id())
            .expect(&format!("node [{}] not found", node_command.node_id()))
            .send(node_command)
            .await
            .unwrap();
    }


    pub async fn add_node_benchmark(&mut self, node_id: NodeId, node_name: String, node_type: String) {
        let mut benchmark_guard = self.benchmark.write().await;
        benchmark_guard.add_node_benchmark(node_id, node_name, node_type);
    }


    pub async fn add_node_cycle_tracker(&mut self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), BacktestStrategyError> {
        let mut benchmark_guard = self.benchmark.write().await;
        benchmark_guard.add_complete_node_cycle(node_id, cycle_tracker)?;
        Ok(())
    }
}
