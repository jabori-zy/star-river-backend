mod command_handler;
mod data_handler;
mod event_handler;
mod node_lifecycle;
mod node_operation;
mod playback_handler;
mod workflow_builder;

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use heartbeat::Heartbeat;
use key::{IndicatorKey, Key, KlineKey};
use sea_orm::DatabaseConnection;
use star_river_core::{
    custom_type::NodeId,
    kline::{Kline, KlineInterval},
};
use strategy_core::{
    event::strategy_event::StrategyRunningLogEvent,
    strategy::{StrategyConfig, context_trait::StrategyMetaDataExt, metadata::StrategyMetadata},
};
use ta_lib::indicator::Indicator;
use tokio::sync::{Mutex, Notify, RwLock, watch};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::strategy_state_machine::{BacktestStrategyRunState, BacktestStrategyStateMachine, backtest_strategy_transition};
use crate::{
    node::{BacktestNode, node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
    virtual_trading_system::{BacktestVts, BacktestVtsContext},
};

pub type BacktestStrategyMetadata =
    StrategyMetadata<BacktestNode, BacktestStrategyStateMachine, BacktestStrategyCommand, BacktestNodeCommand, BacktestNodeEvent>;

#[derive(Debug)]
pub struct BacktestStrategyContext {
    metadata: BacktestStrategyMetadata,
    total_signal_count: Arc<RwLock<i32>>,
    play_index: Arc<RwLock<PlayIndex>>,
    play_index_watch_tx: watch::Sender<PlayIndex>,
    play_index_watch_rx: watch::Receiver<PlayIndex>,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    cancel_play_token: CancellationToken,
    batch_id: Uuid,
    running_log: Arc<RwLock<Vec<StrategyRunningLogEvent>>>,
    execute_over_node_ids: Arc<RwLock<Vec<NodeId>>>,
    execute_over_notify: Arc<Notify>,
    min_interval: KlineInterval,
    base_timeline: Arc<RwLock<Vec<DateTime<Utc>>>>, // base timeline for backtest
    kline_data: Arc<RwLock<HashMap<KlineKey, Vec<Kline>>>>,
    indicator_data: Arc<RwLock<HashMap<IndicatorKey, Vec<Indicator>>>>,
    keys: Arc<RwLock<HashMap<Key, NodeId>>>,
    virtual_trading_system: Arc<Mutex<BacktestVts>>,
}

impl BacktestStrategyContext {
    pub fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let strategy_name = strategy_config.name.clone();
        let state_machine = BacktestStrategyStateMachine::new(
            strategy_name.clone(),
            BacktestStrategyRunState::Created,
            backtest_strategy_transition,
        );

        let metadata = BacktestStrategyMetadata::new("backtest", strategy_config, state_machine, database, heartbeat);

        let (play_index_watch_tx, play_index_watch_rx) = watch::channel::<PlayIndex>(-1);

        let virtual_trading_system = BacktestVts::new(BacktestVtsContext::new());

        Self {
            metadata,
            total_signal_count: Arc::new(RwLock::new(0)),
            play_index: Arc::new(RwLock::new(-1)),
            play_index_watch_tx,
            play_index_watch_rx,
            is_playing: Arc::new(RwLock::new(false)),
            initial_play_speed: Arc::new(RwLock::new(0)),
            cancel_play_token: CancellationToken::new(),
            batch_id: Uuid::new_v4(),
            running_log: Arc::new(RwLock::new(vec![])),
            execute_over_node_ids: Arc::new(RwLock::new(vec![])),
            execute_over_notify: Arc::new(Notify::new()),
            min_interval: KlineInterval::Minutes1,
            base_timeline: Arc::new(RwLock::new(vec![])),
            kline_data: Arc::new(RwLock::new(HashMap::new())),
            indicator_data: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            virtual_trading_system: Arc::new(Mutex::new(virtual_trading_system)),
        }
    }
}

impl StrategyMetaDataExt for BacktestStrategyContext {
    type Node = BacktestNode;
    type StateMachine = BacktestStrategyStateMachine;
    type StrategyCommand = BacktestStrategyCommand;
    type NodeCommand = BacktestNodeCommand;
    type NodeEvent = BacktestNodeEvent;

    fn metadata(&self) -> &StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand, Self::NodeEvent> {
        &self.metadata
    }

    fn metadata_mut(
        &mut self,
    ) -> &mut StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand, Self::NodeEvent> {
        &mut self.metadata
    }
}

// ============================================================================
// 非 Metadata 字段的访问方法
// ============================================================================

impl BacktestStrategyContext {
    // ========================================================================
    // 1. 核心信息 (Core Information)
    // ========================================================================

    /// 获取批次ID
    pub fn batch_id(&self) -> Uuid {
        self.batch_id
    }

    // ========================================================================
    // 2. 执行追踪 (Execution Tracking)
    // ========================================================================

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

    // ========================================================================
    // 3. 播放控制 - 索引 (Playback Index)
    // ========================================================================

    /// 获取播放索引
    pub async fn play_index(&self) -> PlayIndex {
        *self.play_index.read().await
    }

    /// 设置播放索引
    pub async fn set_play_index(&self, index: PlayIndex) {
        *self.play_index.write().await = index;
    }

    /// 获取播放索引监听发送器
    pub fn play_index_watch_tx(&self) -> &watch::Sender<PlayIndex> {
        &self.play_index_watch_tx
    }

    /// 获取播放索引监听接收器克隆
    pub fn play_index_watch_rx(&self) -> watch::Receiver<PlayIndex> {
        self.play_index_watch_rx.clone()
    }

    // ========================================================================
    // 4. 播放控制 - 状态 (Playback State)
    // ========================================================================

    /// 获取播放状态
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.read().await
    }

    /// 设置播放状态
    pub async fn set_is_playing(&self, playing: bool) {
        *self.is_playing.write().await = playing;
    }

    // ========================================================================
    // 5. 播放控制 - 速度 (Playback Speed)
    // ========================================================================

    /// 获取初始播放速度
    pub async fn initial_play_speed(&self) -> u32 {
        *self.initial_play_speed.read().await
    }

    /// 设置初始播放速度
    pub async fn set_initial_play_speed(&self, speed: u32) {
        *self.initial_play_speed.write().await = speed;
    }

    // ========================================================================
    // 6. 播放控制 - 令牌 (Playback Token)
    // ========================================================================

    /// 获取取消播放令牌
    pub fn cancel_play_token(&self) -> CancellationToken {
        self.cancel_play_token.clone()
    }

    // ========================================================================
    // 7. 信号计数 (Signal Count)
    // ========================================================================

    /// 获取信号总数
    pub async fn total_signal_count(&self) -> i32 {
        *self.total_signal_count.read().await
    }

    /// 设置信号总数
    pub async fn set_total_signal_count(&self, count: i32) {
        *self.total_signal_count.write().await = count;
    }

    // ========================================================================
    // 8. 缓存键管理 (Key Management)
    // ========================================================================

    /// 获取缓存键映射
    pub async fn keys(&self) -> HashMap<Key, NodeId> {
        self.keys.read().await.clone()
    }

    /// 添加缓存键
    pub async fn add_key(&self, key: Key, node_id: NodeId) {
        self.keys.write().await.insert(key, node_id);
    }

    // ========================================================================
    // 9. 交易对管理 (Symbol Management)
    // ========================================================================

    /// 获取最小周期
    pub fn min_interval(&self) -> &KlineInterval {
        &self.min_interval
    }

    /// 设置最小周期
    pub fn set_min_interval(&mut self, interval: KlineInterval) {
        self.min_interval = interval;
    }

    // ========================================================================
    // 10. K线数据管理 (Kline Data)
    // ========================================================================

    /// 获取所有K线数据
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

    // ========================================================================
    // 11. 指标数据管理 (Indicator Data)
    // ========================================================================

    /// 获取所有指标数据
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

    // ========================================================================
    // 12. 运行日志管理 (Running Log)
    // ========================================================================

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

    pub fn virtual_trading_system(&self) -> &Arc<Mutex<BacktestVts>> {
        &self.virtual_trading_system
    }
}
