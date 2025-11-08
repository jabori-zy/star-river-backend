// ============================================================================
// 子模块声明
// ============================================================================

mod event_handler;
mod node_handles;
mod benchmark;
mod communicate;

// ============================================================================
// 标准库导入
// ============================================================================

use std::sync::Arc;

// ============================================================================
// 外部 crate 导入
// ============================================================================


use crate::strategy_new::config::BacktestStrategyConfig;
// use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{Mutex, RwLock};
// use virtual_trading::VirtualTradingSystem;
use strategy_core::node::metadata::NodeMetadata;
use crate::strategy_new::PlayIndex;

// ============================================================================
// 当前 crate 内部导入（使用绝对路径）
// ============================================================================

// ============================================================================
// 当前模块内部导入（相对路径）
// ============================================================================

use super::state_machine::StartNodeAction;
use crate::node_list_new::node_state_machine::{NodeRunState, NodeStateTransTrigger};
use crate::node_event::BacktestNodeEvent;
use crate::strategy_command::BacktestStrategyCommand;
use crate::node_command::BacktestNodeCommand;
use super::state_machine::StartNodeStateMachine;
use strategy_core::node::context_trait::{
    NodeMetaDataExt, 
    NodeHandleExt, 
    NodeIdentityExt, 
    NodeBenchmarkExt, NodeStateMachineExt, NodeEventHandlerExt};




pub type StartNodeMetadata = NodeMetadata<StartNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;



#[derive(Debug)]
pub struct StartNodeContext {
    pub metadata: StartNodeMetadata,
    pub node_config: Arc<RwLock<BacktestStrategyConfig>>,
    // pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    // pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl StartNodeContext {
    pub fn new(
        metadata: StartNodeMetadata,
        node_config: Arc<RwLock<BacktestStrategyConfig>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        // virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        // strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
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





    pub async fn send_play_signal(&self) {
        // let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
        // cycle_tracker.start_phase("send_play_signal");
        // let payload = KlinePlayPayload::new(self.play_index());
        // let kline_play_event: StartNodeEvent = KlinePlayEvent::new(
        //     self.node_id().clone(),
        //     self.node_name().clone(),
        //     self.default_output_handle().unwrap().output_handle_id().clone(),
        //     payload,
        // )
        // .into();
        // self.default_output_handle().unwrap().send(kline_play_event.into()).unwrap();
        // cycle_tracker.end_phase("send_play_signal");
        // let completed_tracker = cycle_tracker.end();
        // self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await.unwrap();
    }

    pub async fn init_virtual_trading_system(&self) {
        // let mut virtual_trading_system = self.virtual_trading_system.lock().await;
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
        // let custom_var_configs = {
        //     let node_config_guard = self.node_config.read().await;
        //     node_config_guard.custom_variables.clone()

        // };
        // let (resp_rx, resp_tx) = tokio::sync::oneshot::channel();
        // let init_var_payload = InitCustomVariableCmdPayload::new(custom_var_configs);
        // let init_var_cmd = InitCustomVariableValueCommand::new(self.node_id().clone(),resp_rx, Some(init_var_payload));
        // self.strategy_command_sender().send(init_var_cmd.into()).await.unwrap();
        // let response = resp_tx.await.unwrap();



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
