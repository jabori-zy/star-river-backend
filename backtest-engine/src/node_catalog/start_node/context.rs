// ============================================================================
// 子模块声明
// ============================================================================

mod event_handler;
mod node_handles;

// ============================================================================
// 标准库导入
// ============================================================================

use std::sync::Arc;

use async_trait::async_trait;
use star_river_core::custom_type::NodeId;
use star_river_event::backtest_strategy::node_event::{
    StartNodeEvent,
    start_node_event::{KlinePlayEvent, KlinePlayPayload},
};
use strategy_core::{
    benchmark::node_benchmark::{CompletedCycle, CycleTracker},
    node::{
        context_trait::{
            NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeIdentityExt, NodeMetaDataExt,
            NodeStateMachineExt,
        },
        metadata::NodeMetadata,
    },
};
// use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{Mutex, RwLock};

// ============================================================================
// 当前 crate 内部导入（使用绝对路径）
// ============================================================================

// ============================================================================
// 当前模块内部导入（相对路径）
// ============================================================================
use super::state_machine::{StartNodeAction, StartNodeStateMachine};
// use virtual_trading::VirtualTradingSystem;
use crate::strategy::PlayIndex;
// ============================================================================
// 外部 crate 导入
// ============================================================================
use crate::strategy::strategy_config::BacktestStrategyConfig;
use crate::{
    node::{
        node_command::BacktestNodeCommand,
        node_event::BacktestNodeEvent,
        node_state_machine::{NodeRunState, NodeStateTransTrigger},
    },
    strategy::strategy_command::{BacktestStrategyCommand, InitCustomVarCmdPayload, InitCustomVarValueCommand},
};

pub type StartNodeMetadata = NodeMetadata<StartNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct StartNodeContext {
    pub metadata: StartNodeMetadata,
    pub node_config: Arc<RwLock<BacktestStrategyConfig>>,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl StartNodeContext {
    pub fn new(
        metadata: StartNodeMetadata,
        node_config: Arc<RwLock<BacktestStrategyConfig>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            // virtual_trading_system,
            // strategy_stats,
            play_index_watch_rx,
        }
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }

    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub async fn send_play_signal(&self) {
        let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
        cycle_tracker.start_phase("send_play_signal");
        let payload = KlinePlayPayload::new(self.play_index());
        let kline_play_event: StartNodeEvent = KlinePlayEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
            self.default_output_handle().unwrap().output_handle_id().clone(),
            payload,
        )
        .into();
        self.default_output_handle().unwrap().send(kline_play_event.into()).unwrap();
        cycle_tracker.end_phase("send_play_signal");
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker)
            .await
            .unwrap();
    }

    pub async fn init_virtual_trading_system(&self) {
        // let mut virtual_trading_system = self.virtual_trading_system().lock().await;
        // let node_config = self.node_config.read().await;
        // virtual_trading_system.set_initial_balance(node_config.initial_balance);
        // virtual_trading_system.set_leverage(node_config.leverage as u32);
        // virtual_trading_system.set_fee_rate(node_config.fee_rate);
    }

    pub async fn init_strategy_stats(&self) {
        // let mut strategy_stats = self.strategy_stats.write().await;
        // let node_config = self.node_config.read().await;
        // strategy_stats.set_initial_balance(node_config.initial_balance);
    }

    pub async fn init_custom_variables(&self) {
        let custom_var_configs = {
            let node_config_guard = self.node_config.read().await;
            node_config_guard.custom_variables.clone()
        };
        let (resp_rx, resp_tx) = tokio::sync::oneshot::channel();
        let init_var_payload = InitCustomVarCmdPayload::new(custom_var_configs);
        let init_var_cmd = InitCustomVarValueCommand::new(self.node_id().clone(), resp_rx, init_var_payload);
        self.strategy_command_sender().send(init_var_cmd.into()).await.unwrap();
        let response = resp_tx.await.unwrap();
    }
}

impl NodeMetaDataExt for StartNodeContext {
    type StateMachine = StartNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for StartNodeContext {
    type Error = crate::node::node_error::BacktestNodeError;

    async fn mount_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), Self::Error> {
        crate::node::node_utils::NodeUtils::mount_node_cycle_tracker(node_id, cycle_tracker, self.strategy_command_sender()).await?;
        Ok(())
    }
}
