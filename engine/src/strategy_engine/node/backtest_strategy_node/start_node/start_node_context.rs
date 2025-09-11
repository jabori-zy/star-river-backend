use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::strategy::StrategyCommand;
use event_center::communication::strategy::{BacktestStrategyCommand, GetStartNodeConfigResponse};
use event_center::event::node_event::backtest_node_event::signal_event::{
    KlinePlayEvent, KlinePlayFinishedEvent, KlinePlayFinishedPayload, KlinePlayPayload, SignalEvent,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::Event;
use heartbeat::Heartbeat;

use event_center::communication::strategy::backtest_strategy::command::NodeResetParams;
use event_center::communication::strategy::backtest_strategy::response::NodeResetResponse;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use star_river_core::strategy::BacktestStrategyConfig;
use std::any::Any;
use std::sync::Arc;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use utils::get_utc8_timestamp_millis;
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

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context
            .output_handles
            .get(&format!("start_node_default_output"))
            .unwrap()
            .clone()
    }

    async fn handle_event(&mut self, event: Event) {
        tracing::info!("{}: 收到事件: {:?}", self.base_context.node_id, event);
    }
    async fn handle_node_event(&mut self, message: BacktestNodeEvent) {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {
        // match strategy_inner_event {
        //     StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
        //         // 更新播放索引
        //         self.set_play_index(play_index_update_event.play_index).await;
        //         let strategy_output_handle = self.get_strategy_output_handle();
        //         // 更新完成后，发送索引已更新事件
        //         let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
        //             from_node_id: self.get_node_id().clone(),
        //             from_node_name: self.get_node_name().clone(),
        //             from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
        //             play_index: self.get_play_index().await,
        //             message_timestamp: get_utc8_timestamp_millis(),
        //         }));
        //         strategy_output_handle.send(signal).unwrap();
        //     },
        //     StrategyInnerEvent::NodeReset => {
        //         tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
        //     }
        // }
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        match strategy_command {
            StrategyCommand::BacktestStrategy(BacktestStrategyCommand::GetStartNodeConfig(
                get_start_node_config_params,
            )) => {
                let start_node_config = self.node_config.read().await.clone();

                let response = GetStartNodeConfigResponse::success(
                    self.base_context.node_id.clone(),
                    start_node_config,
                );

                get_start_node_config_params
                    .responder
                    .send(response.into())
                    .unwrap();
            }
            StrategyCommand::BacktestStrategy(BacktestStrategyCommand::NodeReset(
                node_reset_params,
            )) => {
                if self.get_node_id() == &node_reset_params.node_id {
                    let response = NodeResetResponse::success(self.get_node_id().clone());
                    node_reset_params.responder.send(response.into()).unwrap();
                }
            }
        }
    }
}

impl StartNodeContext {
    // 发送k线跳动信号
    pub async fn send_play_signal(&self) {
        let payload = KlinePlayPayload::new(self.get_play_index());
        let kline_play_event: SignalEvent = KlinePlayEvent::new(
            self.base_context.node_id.clone(),
            self.base_context.node_name.clone(),
            self.get_default_output_handle().output_handle_id.clone(),
            payload,
        )
        .into();
        self.get_default_output_handle()
            .send(kline_play_event.into())
            .unwrap();
    }

    // 发送k线播放完毕信号
    pub async fn send_finish_signal(&self, play_index: i32) {
        let payload = KlinePlayFinishedPayload::new(play_index);
        let kline_play_finished_event: SignalEvent = KlinePlayFinishedEvent::new(
            self.base_context.node_id.clone(),
            self.base_context.node_name.clone(),
            self.get_default_output_handle().output_handle_id.clone(),
            payload,
        )
        .into();
        self.get_default_output_handle()
            .send(kline_play_finished_event.into())
            .unwrap();
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

    pub async fn handle_play_index(&self) {
        self.send_play_signal().await;
    }
}
