mod command_handler;
mod data_handler;
mod event_handler;
mod node_lifecycle;
mod node_operation;
mod playback_handler;
mod strategy_operation;
mod workflow_builder;

// std
use std::{
    collections::HashMap,
    sync::Arc,
};

// third-party
use chrono::{DateTime, Utc};
use petgraph::{Directed, Graph, graph::NodeIndex};
use sea_orm::DatabaseConnection;
use tokio::sync::{Mutex, Notify, RwLock, broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// workspace crate
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use event_center::{
    communication::{Command, backtest_strategy::*},
    event::{
        Event,
        node_event::{
            NodeEventTrait,
            backtest_node_event::{
                BacktestNodeEvent, CommonEvent,
                futures_order_node_event::FuturesOrderNodeEvent,
                indicator_node_event::IndicatorNodeEvent,
                kline_node_event::KlineNodeEvent,
                position_node_event::PositionManagementNodeEvent,
            },
        },
        strategy_event::{
            StrategyRunningLogEvent,
            backtest_strategy_event::{BacktestStrategyEvent, PlayFinishedEvent},
        },
    },
    singleton::EventCenterSingleton,
};
use heartbeat::Heartbeat;
use ta_lib::Indicator;
use star_river_core::{
    custom_type::{NodeId, PlayIndex, StrategyId, StrategyName},
    key::{
        Key, KeyTrait,
        key::{IndicatorKey, KlineKey},
    },
    market::{Kline, QuantData},
    order::virtual_order::VirtualOrder,
    position::virtual_position::VirtualPosition,
    strategy::{
        BacktestStrategyConfig, StrategyConfig,
        custom_variable::CustomVariable,
        node_benchmark::CompletedCycle,
        strategy_benchmark::{StrategyBenchmark, StrategyCycleTracker},
        sys_varibale::{SysVariable, SysVariableType},
    },
    strategy_stats::{
        StatsSnapshot,
        event::{StrategyStatsEvent, StrategyStatsEventReceiver},
    },
    system::DateTimeUtc,
    transaction::virtual_transaction::VirtualTransaction,
};
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use virtual_trading::VirtualTradingSystem;

// current crate
use super::{
    node::{BacktestNode, node_context_trait},
    strategy_state_machine::{BacktestStrategyStateMachine, StrategyRunState},
    strategy_utils,
};
use crate::error::strategy_error::BacktestStrategyError;

#[derive(Debug)]
// 回测策略上下文
pub struct BacktestStrategyContext {
    strategy_config: StrategyConfig,
    strategy_id: i32,
    strategy_name: String,                       // 策略名称
    graph: Graph<BacktestNode, (), Directed>,    // 策略的拓扑图
    node_indices: HashMap<String, NodeIndex>,    // 节点索引
    cancel_task_token: CancellationToken,        // 取消令牌
    state_machine: BacktestStrategyStateMachine, // 策略状态机
    database: DatabaseConnection,     // 数据库连接
    heartbeat: Arc<Mutex<Heartbeat>>, // 心跳
    strategy_command_sender: StrategyCommandSender,
    strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>, // 接收节点的命令
    node_command_sender: HashMap<NodeId, NodeCommandSender>,        // 节点命令发送器
    total_signal_count: Arc<RwLock<i32>>,                           // 信号计数
    play_index: Arc<RwLock<i32>>,                                   // 播放索引
    is_playing: Arc<RwLock<bool>>,                                  // 是否正在播放
    initial_play_speed: Arc<RwLock<u32>>,                           // 初始播放速度 （从策略配置中加载）
    cancel_play_token: CancellationToken,                           // 取消播放令牌
    virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,       // 虚拟交易系统
    strategy_stats: Arc<RwLock<BacktestStrategyStats>>,             // 策略统计模块
    strategy_stats_event_receiver: StrategyStatsEventReceiver,      // 策略统计事件接收器
    play_index_watch_tx: tokio::sync::watch::Sender<i32>,    // 播放索引监听器
    play_index_watch_rx: tokio::sync::watch::Receiver<i32>,  // 播放索引监听器
    leaf_node_ids: Vec<NodeId>,                              // 叶子节点id
    execute_over_node_ids: Arc<RwLock<Vec<NodeId>>>,         // 执行完毕的节点id
    execute_over_notify: Arc<Notify>,                        // 已经更新播放索引的节点id通知
    current_time: Arc<RwLock<DateTime<Utc>>>,                // 当前时间
    batch_id: Uuid,                                          // 回测批次id
    running_log: Arc<RwLock<Vec<StrategyRunningLogEvent>>>,  // 运行日志
    keys: Arc<RwLock<HashMap<Key, NodeId>>>,                 // 缓存键 -> 其所属节点id
    min_interval_symbols: Vec<KlineKey>,                     // 最小周期交易对
    kline_data: Arc<RwLock<HashMap<KlineKey, Vec<Kline>>>>,  // 所有k线数据
    indicator_data: Arc<RwLock<HashMap<IndicatorKey, Vec<Indicator>>>>, // 所有指标数据
    custom_variable: Arc<RwLock<HashMap<String, CustomVariable>>>, // var_name -> CustomVariable
    sys_variable: Arc<RwLock<HashMap<SysVariableType, SysVariable>>>, // var_name -> SysVariable
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
            state_machine: BacktestStrategyStateMachine::new(strategy_id, strategy_name, StrategyRunState::Created),
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

//     pub async fn get_keys(&self) -> HashMap<Key, NodeId> {
//         self.keys.read().await.clone()
//     }

//     pub fn set_state_machine(&mut self, state_machine: BacktestStrategyStateMachine) {
//         self.state_machine = state_machine;
//     }

//     pub async fn get_current_time(&self) -> DateTime<Utc> {
//         self.current_time.read().await.clone()
//     }

//     pub async fn set_current_time(&mut self, current_time: DateTime<Utc>) {
//         *self.current_time.write().await = current_time;
//     }

//     pub async fn get_running_log(&self) -> Vec<StrategyRunningLogEvent> {
//         self.running_log.read().await.clone()
//     }

//     pub async fn add_running_log(&mut self, running_log: StrategyRunningLogEvent) {
//         self.running_log.write().await.push(running_log);
//     }

//     pub async fn reset_running_log(&mut self) {
//         self.running_log.write().await.clear();
//     }

//     pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
//         self.min_interval_symbols = min_interval_symbols;
//     }

//     pub fn get_min_interval_symbols(&self) -> Vec<KlineKey> {
//         self.min_interval_symbols.clone()
//     }

//     // pub fn set_all_node_output_handles(&mut self, all_node_output_handles: Vec<NodeOutputHandle>) {
//     //     self.all_node_output_handles = all_node_output_handles;
//     // }

//     pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
//         self.leaf_node_ids = leaf_node_ids;
//     }

//     // pub fn get_all_node_output_handles(&self) -> Vec<NodeOutputHandle> {
//     //     self.all_node_output_handles.clone()
//     // }

//     pub fn get_cancel_task_token(&self) -> CancellationToken {
//         self.cancel_task_token.clone()
//     }

//     pub fn get_strategy_command_receiver(&self) -> Arc<Mutex<StrategyCommandReceiver>> {
//         self.strategy_command_receiver.clone()
//     }

//     // 添加节点命令发送器
//     pub async fn add_node_command_sender(&mut self, node_id: NodeId, sender: NodeCommandSender) {
//         self.node_command_sender.insert(node_id, sender);
//     }

//     pub async fn send_node_command(&self, node_command: BacktestNodeCommand) {
//         self.node_command_sender
//             .get(&node_command.node_id())
//             .expect(&format!("node [{}] not found", node_command.node_id()))
//             .send(node_command)
//             .await
//             .unwrap();
//     }

//     pub async fn add_node_benchmark(&mut self, node_id: NodeId, node_name: String, node_type: String) {
//         let mut benchmark_guard = self.benchmark.write().await;
//         benchmark_guard.add_node_benchmark(node_id, node_name, node_type);
//     }

//     // pub async fn add_node_cycle_tracker(&mut self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), BacktestStrategyError> {
//     //     let mut benchmark_guard = self.benchmark.write().await;
//     //     benchmark_guard.add_complete_node_cycle(node_id, cycle_tracker)?;
//     //     Ok(())
//     // }
}

// ============================================================================
// 1. 核心信息 (Core Information)
// 包括: 基础信息、状态机、外部依赖
// ============================================================================
impl BacktestStrategyContext {
    // --- 基础信息 ---

    /// 获取策略ID
    pub fn strategy_id(&self) -> StrategyId {
        self.strategy_id
    }

    /// 获取策略名称引用
    pub fn strategy_name(&self) -> &StrategyName {
        &self.strategy_name
    }

    /// 获取策略配置引用
    pub fn strategy_config(&self) -> &StrategyConfig {
        &self.strategy_config
    }

    /// 获取批次ID
    pub fn batch_id(&self) -> Uuid {
        self.batch_id
    }

    // --- 状态机 ---

    /// 获取状态机引用
    pub fn state_machine(&self) -> &BacktestStrategyStateMachine {
        &self.state_machine
    }

    /// 设置状态机
    pub fn set_state_machine(&mut self, state_machine: BacktestStrategyStateMachine) {
        self.state_machine = state_machine;
    }

    // --- 外部依赖 ---

    /// 获取数据库连接引用
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }

    /// 获取心跳的Arc引用
    pub fn heartbeat(&self) -> Arc<Mutex<Heartbeat>> {
        self.heartbeat.clone()
    }
}

// ============================================================================
// 2. 工作流管理 (Workflow Management)
// 包括: 图结构、节点索引、执行追踪
// ============================================================================
impl BacktestStrategyContext {
    // --- 图结构 ---

    /// 获取图的可变引用
    pub fn graph_mut(&mut self) -> &mut Graph<BacktestNode, (), Directed> {
        &mut self.graph
    }

    /// 获取图的不可变引用
    pub fn graph(&self) -> &Graph<BacktestNode, (), Directed> {
        &self.graph
    }

    pub fn node(&self, node_index: NodeIndex) -> Option<&BacktestNode> {
        self.graph.node_weight(node_index)
    }

    pub fn node_mut(&mut self, node_index: NodeIndex) -> Option<&mut BacktestNode> {
        self.graph.node_weight_mut(node_index)
    }

    /// 获取节点索引映射的可变引用
    pub fn node_indices_mut(&mut self) -> &mut HashMap<String, NodeIndex> {
        &mut self.node_indices
    }

    /// 获取节点索引映射的不可变引用
    pub fn node_indices(&self) -> &HashMap<String, NodeIndex> {
        &self.node_indices
    }

    /// 添加节点
    pub async fn add_node(&mut self, node: BacktestNode) -> NodeIndex {
        let node_id = node.node_id().await;
        let node_index = self.graph.add_node(node);
        self.node_indices.insert(node_id.to_string(), node_index);
        node_index
    }

    /// 获取叶子节点ID列表
    pub fn leaf_node_ids(&self) -> &[NodeId] {
        &self.leaf_node_ids
    }

    /// 设置叶子节点ID列表
    pub fn set_leaf_node_ids(&mut self, leaf_node_ids: Vec<NodeId>) {
        self.leaf_node_ids = leaf_node_ids;
    }

    // --- 执行追踪 ---

    /// 获取已执行完毕的节点ID列表
    pub async fn execute_over_node_ids(&self) -> Vec<NodeId> {
        self.execute_over_node_ids.read().await.clone()
    }

    /// 添加执行完毕的节点ID
    pub async fn add_execute_over_node_id(&self, node_id: NodeId) {
        self.execute_over_node_ids.write().await.push(node_id);
    }

    /// 清空执行完毕的节点ID列表
    pub async fn clear_execute_over_node_ids(&self) {
        self.execute_over_node_ids.write().await.clear();
    }

    /// 获取执行完毕通知器
    pub fn execute_over_notify(&self) -> Arc<Notify> {
        self.execute_over_notify.clone()
    }
}

// ============================================================================
// 3. 通信与控制 (Communication & Control)
// 包括: 命令通信、任务控制
// ============================================================================
impl BacktestStrategyContext {
    // --- 策略级通信 ---

    /// 获取策略命令发送器
    pub fn strategy_command_sender(&self) -> &StrategyCommandSender {
        &self.strategy_command_sender
    }

    /// 获取策略命令接收器
    pub fn strategy_command_receiver(&self) -> Arc<Mutex<StrategyCommandReceiver>> {
        self.strategy_command_receiver.clone()
    }

    /// 获取策略统计事件接收器的克隆
    pub fn strategy_stats_event_receiver(&self) -> StrategyStatsEventReceiver {
        self.strategy_stats_event_receiver.resubscribe()
    }

    // --- 节点级通信 ---

    /// 添加节点命令发送器
    pub fn add_node_command_sender(&mut self, node_id: NodeId, sender: NodeCommandSender) {
        self.node_command_sender.insert(node_id, sender);
    }

    /// 发送节点命令
    pub async fn send_node_command(&self, node_command: BacktestNodeCommand) -> Result<(), BacktestStrategyError> {
        let node_id = node_command.node_id();
        self.node_command_sender
            .get(&node_id)
            .expect(&format!("node [{}] not found", node_id))
            .send(node_command)
            .await
            .unwrap();
        Ok(())
    }

    // --- 任务控制 ---

    /// 获取取消任务令牌
    pub fn cancel_task_token(&self) -> CancellationToken {
        self.cancel_task_token.clone()
    }

    /// 获取取消播放令牌
    pub fn cancel_play_token(&self) -> CancellationToken {
        self.cancel_play_token.clone()
    }
}

// ============================================================================
// 4. 回放控制 (Playback Control)
// 包括: 播放状态、索引、速度、信号计数
// ============================================================================
impl BacktestStrategyContext {
    // --- 播放索引 ---

    /// 获取播放索引
    pub async fn play_index(&self) -> i32 {
        *self.play_index.read().await
    }

    /// 设置播放索引
    pub async fn set_play_index(&self, index: PlayIndex) {
        *self.play_index.write().await = index;
    }

    /// 获取播放索引监听发送器
    pub fn play_index_watch_tx(&self) -> &tokio::sync::watch::Sender<i32> {
        &self.play_index_watch_tx
    }

    /// 获取播放索引监听接收器克隆
    pub fn play_index_watch_rx(&self) -> tokio::sync::watch::Receiver<PlayIndex> {
        self.play_index_watch_rx.clone()
    }

    // --- 播放状态 ---

    /// 获取播放状态
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.read().await
    }

    /// 设置播放状态
    pub async fn set_is_playing(&self, playing: bool) {
        *self.is_playing.write().await = playing;
    }

    // --- 播放速度 ---

    /// 获取初始播放速度
    pub async fn initial_play_speed(&self) -> u32 {
        *self.initial_play_speed.read().await
    }

    /// 设置初始播放速度
    pub async fn set_initial_play_speed(&self, speed: u32) {
        *self.initial_play_speed.write().await = speed;
    }

    // --- 信号计数 ---

    /// 获取信号总数
    pub async fn total_signal_count(&self) -> i32 {
        *self.total_signal_count.read().await
    }

    /// 设置信号总数
    pub async fn set_total_signal_count(&self, count: i32) {
        *self.total_signal_count.write().await = count;
    }

    // --- 时间管理 ---

    /// 获取当前时间
    pub async fn current_time(&self) -> DateTime<Utc> {
        *self.current_time.read().await
    }

    /// 设置当前时间
    pub async fn set_current_time(&self, time: DateTime<Utc>) {
        *self.current_time.write().await = time;
    }
}

// ============================================================================
// 5. 数据管理 (Data Management)
// 包括: K线数据、指标数据、变量、缓存键
// ============================================================================
impl BacktestStrategyContext {
    // --- 缓存键管理 ---

    /// 获取缓存键映射
    pub async fn keys(&self) -> HashMap<Key, NodeId> {
        self.keys.read().await.clone()
    }

    /// 添加缓存键
    pub async fn add_key(&self, key: Key, node_id: NodeId) {
        self.keys.write().await.insert(key, node_id);
    }

    /// 获取最小周期交易对列表
    pub fn min_interval_symbols(&self) -> &[KlineKey] {
        &self.min_interval_symbols
    }

    /// 设置最小周期交易对列表
    pub fn set_min_interval_symbols(&mut self, symbols: Vec<KlineKey>) {
        self.min_interval_symbols = symbols;
    }

    // --- K线数据 ---

    /// 获取K线数据
    pub async fn kline_data(&self) -> HashMap<KlineKey, Vec<Kline>> {
        self.kline_data.read().await.clone()
    }

    /// 获取指定K线数据
    pub async fn get_kline_data(&self, key: &KlineKey) -> Option<Vec<Kline>> {
        self.kline_data.read().await.get(key).cloned()
    }

    /// 设置K线数据
    pub async fn set_kline_data(&self, key: KlineKey, data: Vec<Kline>) {
        self.kline_data.write().await.insert(key, data);
    }

    /// 获取K线数据的Arc引用
    pub fn kline_data_arc(&self) -> Arc<RwLock<HashMap<KlineKey, Vec<Kline>>>> {
        self.kline_data.clone()
    }

    // --- 指标数据 ---

    /// 获取指标数据
    pub async fn indicator_data(&self) -> HashMap<IndicatorKey, Vec<Indicator>> {
        self.indicator_data.read().await.clone()
    }

    /// 获取指定指标数据
    pub async fn get_indicator_data(&self, key: &IndicatorKey) -> Option<Vec<Indicator>> {
        self.indicator_data.read().await.get(key).cloned()
    }

    /// 设置指标数据
    pub async fn set_indicator_data(&self, key: IndicatorKey, data: Vec<Indicator>) {
        self.indicator_data.write().await.insert(key, data);
    }

    /// 获取指标数据的Arc引用
    pub fn indicator_data_arc(&self) -> Arc<RwLock<HashMap<IndicatorKey, Vec<Indicator>>>> {
        self.indicator_data.clone()
    }

    // --- 自定义变量 ---

    /// 获取自定义变量
    pub async fn custom_variables(&self) -> HashMap<String, CustomVariable> {
        self.custom_variable.read().await.clone()
    }

    /// 获取指定自定义变量
    pub async fn get_custom_variable(&self, name: &str) -> Option<CustomVariable> {
        self.custom_variable.read().await.get(name).cloned()
    }

    /// 设置自定义变量
    pub async fn set_custom_variable(&self, name: String, variable: CustomVariable) {
        self.custom_variable.write().await.insert(name, variable);
    }

    /// 获取自定义变量的Arc引用
    pub fn custom_variable_arc(&self) -> Arc<RwLock<HashMap<String, CustomVariable>>> {
        self.custom_variable.clone()
    }

    // --- 系统变量 ---

    /// 获取系统变量
    pub async fn sys_variables(&self) -> HashMap<SysVariableType, SysVariable> {
        self.sys_variable.read().await.clone()
    }

    /// 获取指定系统变量
    pub async fn get_sys_variable(&self, var_type: &SysVariableType) -> Option<SysVariable> {
        self.sys_variable.read().await.get(var_type).cloned()
    }

    /// 设置系统变量
    pub async fn set_sys_variable(&self, var_type: SysVariableType, variable: SysVariable) {
        self.sys_variable.write().await.insert(var_type, variable);
    }

    /// 获取系统变量的Arc引用
    pub fn sys_variable_arc(&self) -> Arc<RwLock<HashMap<SysVariableType, SysVariable>>> {
        self.sys_variable.clone()
    }
}

// ============================================================================
// 6. 交易与统计 (Trading & Statistics)
// 包括: 虚拟交易系统、策略统计、基准测试、日志
// ============================================================================
impl BacktestStrategyContext {
    // --- 虚拟交易系统 ---

    /// 获取虚拟交易系统的Arc引用
    pub fn virtual_trading_system(&self) -> &Arc<Mutex<VirtualTradingSystem>> {
        &self.virtual_trading_system
    }

    // --- 策略统计 ---

    /// 获取策略统计的Arc引用
    pub fn strategy_stats(&self) -> Arc<RwLock<BacktestStrategyStats>> {
        self.strategy_stats.clone()
    }

    // --- 基准测试 ---

    /// 获取基准测试的Arc引用
    pub fn benchmark(&self) -> Arc<RwLock<StrategyBenchmark>> {
        self.benchmark.clone()
    }

    /// 添加节点基准测试
    pub async fn add_node_benchmark(&self, node_id: NodeId, node_name: String, node_type: String) {
        self.benchmark.write().await.add_node_benchmark(node_id, node_name, node_type);
    }

    /// 添加节点周期追踪器
    pub async fn add_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), BacktestStrategyError> {
        self.benchmark.write().await.add_complete_node_cycle(node_id, cycle_tracker)?;
        Ok(())
    }

    /// 获取周期追踪器
    // pub async fn cycle_tracker(&self) -> Option<&StrategyCycleTracker> {
    //     self.cycle_tracker.read().await.as_ref()
    // }

    /// 设置周期追踪器
    pub async fn set_cycle_tracker(&self, tracker: Option<StrategyCycleTracker>) {
        *self.cycle_tracker.write().await = tracker;
    }

    /// 获取周期追踪器的Arc引用
    pub fn cycle_tracker_arc(&self) -> Arc<RwLock<Option<StrategyCycleTracker>>> {
        self.cycle_tracker.clone()
    }

    // --- 运行日志 ---

    /// 获取运行日志
    pub async fn running_log(&self) -> Vec<StrategyRunningLogEvent> {
        self.running_log.read().await.clone()
    }

    /// 添加运行日志
    pub async fn add_running_log(&self, log: StrategyRunningLogEvent) {
        self.running_log.write().await.push(log);
    }

    /// 清空运行日志
    pub async fn clear_running_log(&self) {
        self.running_log.write().await.clear();
    }
}
