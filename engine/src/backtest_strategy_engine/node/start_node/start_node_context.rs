use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::{Command, Response};
use event_center::communication::backtest_strategy::{BacktestNodeCommand, GetStartNodeConfigResponse, InitCustomVariableValueCmdPayload, InitCustomVariableValueCommand};
use event_center::communication::backtest_strategy::{GetStartNodeConfigRespPayload, NodeResetResponse};
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::start_node_event::{KlinePlayEvent, KlinePlayPayload, StartNodeEvent};
use heartbeat::Heartbeat;
use star_river_core::strategy::BacktestStrategyConfig;

use std::any::Any;
use std::sync::Arc;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use virtual_trading::VirtualTradingSystem;

#[derive(Debug, Clone)]
pub struct StartNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub node_config: Arc<RwLock<BacktestStrategyConfig>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    pub strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
}

#[async_trait]
impl BacktestNodeContextTrait for StartNodeContext {
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> &NodeOutputHandle {
        self.base_context.output_handles.get(&format!("start_node_default_output")).unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
    }
    async fn handle_node_event(&mut self, message: BacktestNodeEvent) {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        match node_command {
            BacktestNodeCommand::GetStartNodeConfig(cmd) => {
                let start_node_config = self.node_config.read().await.clone();

                let payload = GetStartNodeConfigRespPayload::new(start_node_config);
                let response = GetStartNodeConfigResponse::success(self.get_node_id().clone(), Some(payload));
                cmd.respond(response);
            }
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
        }
    }
}

impl StartNodeContext {
    // 发送k线跳动信号
    pub async fn send_play_signal(&self) {
        let payload = KlinePlayPayload::new(self.get_play_index());
        let kline_play_event: StartNodeEvent = KlinePlayEvent::new(
            self.base_context.node_id.clone(),
            self.base_context.node_name.clone(),
            self.get_default_output_handle().output_handle_id.clone(),
            payload,
        )
        .into();
        self.get_default_output_handle().send(kline_play_event.into()).unwrap();
    }

    // 发送k线播放完毕信号
    // pub async fn send_finish_signal(&self, play_index: i32) {
    //     let payload = KlinePlayFinishedPayload::new(play_index);
    //     let play_finished_event: StartNodeEvent = KlinePlayFinishedEvent::new(
    //         self.base_context.node_id.clone(),
    //         self.base_context.node_name.clone(),
    //         self.get_default_output_handle().output_handle_id.clone(),
    //         payload,
    //     )
    //     .into();
    //     self.get_default_output_handle()
    //         .send(play_finished_event.into())
    //         .unwrap();
    // }

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

    pub async fn handle_play_index(&self) {
        self.send_play_signal().await;
    }



    pub async fn init_custom_variables(&self) {
        let custom_var_configs = {
            let node_config_guard = self.node_config.read().await;
            node_config_guard.custom_variables.clone()

        };
        let (resp_rx, resp_tx) = tokio::sync::oneshot::channel();
        let init_var_payload = InitCustomVariableValueCmdPayload::new(custom_var_configs);
        let init_var_cmd = InitCustomVariableValueCommand::new(self.get_node_id().clone(),resp_rx, Some(init_var_payload));
        self.get_strategy_command_sender().send(init_var_cmd.into()).await.unwrap();
        let response = resp_tx.await.unwrap();



    }
}
