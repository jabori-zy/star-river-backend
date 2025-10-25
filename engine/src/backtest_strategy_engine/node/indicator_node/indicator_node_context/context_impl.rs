use super::{
    IndicatorNodeContext, KlineNodeEvent,BacktestBaseNodeContext,BacktestNodeContextTrait,NodeOutputHandle
};

use async_trait::async_trait;
use event_center::communication::Command;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse};
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::key::KeyTrait;


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
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    let config_kline = self.node_config.exchange_mode_config.as_ref().unwrap().selected_symbol.clone();
                        if config_kline.symbol != kline_update_event.kline_key.get_symbol() || 
                            config_kline.interval != kline_update_event.kline_key.get_interval() {
                            return;
                        }
                        self.handle_kline_update(kline_update_event).await;
                }
            }
            _ => {}
        }
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    self.kline_value.clear();
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}
