// ============================================================================
// 子模块声明
// ============================================================================

mod event_handler;
mod node_handles;

// ============================================================================
// 标准库导入
// ============================================================================

use std::sync::Arc;

// ============================================================================
// 外部 crate 导入
// ============================================================================

use event_center::communication::backtest_strategy::{InitCustomVariableCmdPayload, InitCustomVariableValueCommand};
use event_center::event::node_event::backtest_node_event::start_node_event::{KlinePlayEvent, KlinePlayPayload, StartNodeEvent};
use star_river_core::strategy::BacktestStrategyConfig;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::{Mutex, RwLock};
use virtual_trading::VirtualTradingSystem;
use star_river_core::strategy::node_benchmark::CycleTracker;

// ============================================================================
// 当前 crate 内部导入（使用绝对路径）
// ============================================================================

use crate::node::base_context::NodeBaseContext;
use crate::node::node_context_trait::{NodeBaseContextTrait, NodeCommunication, NodeIdentity, NodeHandleTrait, NodePlayback, NodeBenchmark};

// ============================================================================
// 当前模块内部导入（相对路径）
// ============================================================================

use super::state_machine::StartNodeAction;

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: NodeBaseContext<StartNodeAction>,
    pub node_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
}

impl StartNodeContext {
    pub fn new(
        base_context: NodeBaseContext<StartNodeAction>,
        node_config: Arc<RwLock<BacktestStrategyConfig>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
    ) -> Self {

        Self {
            base_context,
            node_config,
            virtual_trading_system,
            strategy_stats,
        }
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
        self.add_node_cycle_tracker(self.node_id().clone(), completed_tracker).await.unwrap();
    }

    pub async fn init_virtual_trading_system(&self) {
        let mut virtual_trading_system = self.virtual_trading_system.lock().await;
        let node_config = self.node_config.read().await;
        virtual_trading_system.set_initial_balance(node_config.initial_balance);
        virtual_trading_system.set_leverage(node_config.leverage as u32);
        virtual_trading_system.set_fee_rate(node_config.fee_rate);
    }

    pub async fn init_strategy_stats(&self) {
        let mut strategy_stats = self.strategy_stats.write().await;
        let node_config = self.node_config.read().await;
        strategy_stats.set_initial_balance(node_config.initial_balance);
    }



    pub async fn init_custom_variables(&self) {
        let custom_var_configs = {
            let node_config_guard = self.node_config.read().await;
            node_config_guard.custom_variables.clone()

        };
        let (resp_rx, resp_tx) = tokio::sync::oneshot::channel();
        let init_var_payload = InitCustomVariableCmdPayload::new(custom_var_configs);
        let init_var_cmd = InitCustomVariableValueCommand::new(self.node_id().clone(),resp_rx, Some(init_var_payload));
        self.strategy_command_sender().send(init_var_cmd.into()).await.unwrap();
        let response = resp_tx.await.unwrap();



    }
}

impl NodeBaseContextTrait<StartNodeAction> for StartNodeContext {
    fn base_context(&self) -> &NodeBaseContext<StartNodeAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut NodeBaseContext<StartNodeAction> {
        &mut self.base_context
    }
}
