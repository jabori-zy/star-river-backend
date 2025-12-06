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
    event::node_common_event::NodeRunningLogEvent,
    strategy::{StrategyConfig, context_trait::StrategyMetaDataExt, metadata::StrategyMetadata},
};
use ta_lib::indicator::Indicator;
use tokio::sync::{Mutex, Notify, RwLock, watch};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{
    signal_generator::SignalGenerator,
    strategy_state_machine::{BacktestStrategyRunState, BacktestStrategyStateMachine, backtest_strategy_transition},
};
use crate::{
    node::{BacktestNode, node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::{strategy_command::BacktestStrategyCommand, strategy_error::BacktestStrategyError},
    strategy_stats::BacktestStrategyStats,
    virtual_trading_system::{BacktestVts, BacktestVtsContext},
};
pub type BacktestStrategyMetadata = StrategyMetadata<
    BacktestNode,
    BacktestStrategyStateMachine,
    BacktestStrategyCommand,
    BacktestNodeCommand,
    BacktestNodeEvent,
    BacktestStrategyStats,
>;

#[derive(Debug)]
pub struct BacktestStrategyContext {
    metadata: BacktestStrategyMetadata,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    cancel_play_token: CancellationToken,
    pub(crate) batch_id: Uuid,
    running_log: Arc<RwLock<Vec<NodeRunningLogEvent>>>,
    execute_over_node_ids: Arc<RwLock<Vec<NodeId>>>,
    execute_over_notify: Arc<Notify>,
    pub(crate) min_interval: KlineInterval,
    pub(crate) kline_data: Arc<RwLock<HashMap<KlineKey, Vec<Kline>>>>,
    pub(crate) indicator_data: Arc<RwLock<HashMap<IndicatorKey, Vec<Indicator>>>>,
    keys: Arc<RwLock<HashMap<Key, NodeId>>>,
    pub(crate) vts: Arc<BacktestVts>,
    pub(crate) signal_generator: Arc<Mutex<SignalGenerator>>,
}

impl BacktestStrategyContext {
    pub fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let strategy_name = strategy_config.name.clone();
        let state_machine = BacktestStrategyStateMachine::new(
            strategy_name.clone(),
            BacktestStrategyRunState::Created,
            backtest_strategy_transition,
        );

        let (strategy_time_watch_tx, strategy_time_watch_rx) = watch::channel::<DateTime<Utc>>(Utc::now());

        let vts = Arc::new(BacktestVts::new(BacktestVtsContext::new(strategy_time_watch_rx)));
        let strategy_stats = BacktestStrategyStats::new(
            strategy_config.id,
            strategy_name,
            strategy_time_watch_tx.subscribe(),
            Arc::clone(&vts),
        );

        let metadata = BacktestStrategyMetadata::new(
            strategy_config,
            state_machine,
            database,
            heartbeat,
            strategy_stats,
            strategy_time_watch_tx,
        );

        Self {
            metadata,
            is_playing: Arc::new(RwLock::new(false)),
            initial_play_speed: Arc::new(RwLock::new(0)),
            cancel_play_token: CancellationToken::new(),
            batch_id: Uuid::new_v4(),
            running_log: Arc::new(RwLock::new(vec![])),
            execute_over_node_ids: Arc::new(RwLock::new(vec![])),
            execute_over_notify: Arc::new(Notify::new()),
            min_interval: KlineInterval::Months1,
            kline_data: Arc::new(RwLock::new(HashMap::new())),
            indicator_data: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            vts,
            signal_generator: Arc::new(Mutex::new(SignalGenerator::new())),
        }
    }
}

impl StrategyMetaDataExt for BacktestStrategyContext {
    type Node = BacktestNode;
    type StateMachine = BacktestStrategyStateMachine;
    type StrategyCommand = BacktestStrategyCommand;
    type NodeCommand = BacktestNodeCommand;
    type NodeEvent = BacktestNodeEvent;
    type StrategyStats = BacktestStrategyStats;
    type Error = BacktestStrategyError;

    fn metadata(
        &self,
    ) -> &StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand, Self::NodeEvent, Self::StrategyStats>
    {
        &self.metadata
    }

    fn metadata_mut(
        &mut self,
    ) -> &mut StrategyMetadata<Self::Node, Self::StateMachine, Self::StrategyCommand, Self::NodeCommand, Self::NodeEvent, Self::StrategyStats>
    {
        &mut self.metadata
    }
}

// ============================================================================
// Access methods for non-Metadata fields
// ============================================================================

impl BacktestStrategyContext {
    // ========================================================================
    // 2. Execution Tracking
    // ========================================================================

    /// Get list of node IDs that have finished execution
    pub async fn execute_over_node_ids(&self) -> Vec<NodeId> {
        self.execute_over_node_ids.read().await.clone()
    }

    /// Add a node ID that has finished execution
    pub async fn add_execute_over_node_id(&self, node_id: NodeId) {
        self.execute_over_node_ids.write().await.push(node_id);
    }

    /// Clear the list of finished node IDs
    pub async fn clear_execute_over_node_ids(&self) {
        self.execute_over_node_ids.write().await.clear();
    }

    /// Get the execution finished notifier
    pub fn execute_over_notify(&self) -> Arc<Notify> {
        self.execute_over_notify.clone()
    }

    // ========================================================================
    // 4. Playback Control - State
    // ========================================================================

    /// Get playback state
    pub async fn is_playing(&self) -> bool {
        *self.is_playing.read().await
    }

    /// Set playback state
    pub async fn set_is_playing(&self, playing: bool) {
        *self.is_playing.write().await = playing;
    }

    // ========================================================================
    // 5. Playback Control - Speed
    // ========================================================================

    /// Get initial playback speed
    pub async fn initial_play_speed(&self) -> u32 {
        *self.initial_play_speed.read().await
    }

    /// Set initial playback speed
    pub async fn set_initial_play_speed(&self, speed: u32) {
        *self.initial_play_speed.write().await = speed;
    }

    // ========================================================================
    // 6. Playback Control - Token
    // ========================================================================

    /// Get cancellation token for playback
    pub fn cancel_play_token(&self) -> CancellationToken {
        self.cancel_play_token.clone()
    }

    // ========================================================================
    // 8. Key Management
    // ========================================================================

    /// Get cache key mapping
    pub async fn keys(&self) -> HashMap<Key, NodeId> {
        self.keys.read().await.clone()
    }

    /// Add a cache key
    pub async fn add_key(&self, key: Key, node_id: NodeId) {
        self.keys.write().await.insert(key, node_id);
    }

    // ========================================================================
    // 9. Symbol Management
    // ========================================================================

    /// Set minimum interval
    pub fn set_min_interval(&mut self, interval: KlineInterval) {
        self.min_interval = interval;
    }

    // ========================================================================
    // 12. Running Log Management
    // ========================================================================

    /// Get running log
    pub async fn running_log(&self) -> Vec<NodeRunningLogEvent> {
        self.running_log.read().await.clone()
    }

    /// Add running log entry
    pub async fn add_running_log(&self, log: NodeRunningLogEvent) {
        self.running_log.write().await.push(log);
    }

    /// Clear running log
    pub async fn clear_running_log(&self) {
        self.running_log.write().await.clear();
    }
}
