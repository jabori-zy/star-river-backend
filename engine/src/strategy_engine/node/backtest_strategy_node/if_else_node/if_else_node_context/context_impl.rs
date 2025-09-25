use super::IfElseNodeContext;

use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::backtest_strategy::StrategyCommand;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse};
use event_center::communication::Command;
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};
use event_center::event::node_event::backtest_node_event::indicator_node_event::IndicatorNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::variable_node_event::VariableNodeEvent;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for IfElseNodeContext {
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
        let else_output_handle_id = format!("{}_else_output", self.get_node_id());
        self.base_context.output_handles.get(&else_output_handle_id).unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::debug!("{}: 收到节点事件: {:?}", self.get_node_id(), node_event);

        // 检查是否需要更新接收事件
        let should_update = match &node_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                self.get_play_index() == indicator_update_event.play_index
            }
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                self.get_play_index() == kline_update_event.play_index
            }
            BacktestNodeEvent::VariableNode(VariableNodeEvent::SysVariableUpdated(_)) => true,
            _ => false,
        };

        if should_update {
            self.update_received_event(node_event.clone());
        }

        match &node_event {
            BacktestNodeEvent::Common(signal_event) => match signal_event {
                CommonEvent::Trigger(_) => {
                    tracing::debug!("{}: 接收到trigger事件。 不需要逻辑判断", self.get_node_id());

                    self.handle_trigger_event().await;
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn handle_strategy_inner_event(&mut self, _strategy_inner_event: StrategyInnerEvent) {}

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}
