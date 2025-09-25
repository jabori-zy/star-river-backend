use super::IndicatorNodeContext;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse};
use event_center::communication::Command;
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for IndicatorNodeContext {
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
        let default_output_handle_id = format!("{}_default_output", self.base_context.node_id);
        self.base_context.output_handles.get(&default_output_handle_id).unwrap()
    }

    async fn handle_engine_event(&mut self, _event: Event) {}

    async fn handle_node_event(&mut self, message: BacktestNodeEvent) {
        match message {
            BacktestNodeEvent::KlineNode(kline_event) => {
                self.handle_kline_update(kline_event).await;
            }
            _ => {}
        }
    }

    async fn handle_strategy_inner_event(&mut self, _strategy_inner_event: StrategyInnerEvent) {}

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
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
