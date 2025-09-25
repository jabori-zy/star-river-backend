use super::KlineNodeContext;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse, StrategyCommand};
use event_center::communication::Command;
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::start_node_event::StartNodeEvent;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for KlineNodeContext {
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
        let node_id = self.base_context.node_id.clone();
        self.base_context
            .output_handles
            .get(&format!("{}_default_output", node_id))
            .unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, node_event);
        // 收到消息之后，获取对应index的k线数据

        match node_event {
            BacktestNodeEvent::StartNode(start_node_event) => match start_node_event {
                StartNodeEvent::KlinePlay(play_event) => {
                    self.send_kline(play_event).await;
                }
            },
            _ => {}
        }
    }

    // 处理策略内部事件
    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {
        match strategy_inner_event {
            // StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
            //     // 更新k线缓存索引
            //     self.set_play_index(play_index_update_event.play_index).await;
            //     let strategy_output_handle = self.get_strategy_output_handle();
            //     let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
            //         from_node_id: self.get_node_id().clone(),
            //         from_node_name: self.get_node_name().clone(),
            //         from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
            //         play_index: self.get_play_index().await,
            //         message_timestamp: get_utc8_timestamp_millis(),
            //     }));
            //     if let Err(e) = strategy_output_handle.send(signal) {
            //         tracing::error!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "send event failed: {}", e);
            //     }

            // }
            StrategyInnerEvent::NodeReset => {
                // tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    self.handle_node_reset().await;
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}
