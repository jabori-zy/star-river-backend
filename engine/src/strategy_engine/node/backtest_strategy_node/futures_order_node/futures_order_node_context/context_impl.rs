use super::FuturesOrderNodeContext;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::strategy::StrategyCommand;
use event_center::communication::strategy::backtest_strategy::command::BacktestNodeCommand;
use event_center::communication::strategy::backtest_strategy::response::NodeResetResponse;
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for FuturesOrderNodeContext {
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
        self.base_context
            .output_handles
            .get(&format!("{}_default_output", self.get_node_id()))
            .unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::debug!("{}: 接收到节点事件: {:?}", self.get_node_id(), node_event);
        // match node_event {
        //     NodeEvent::Signal(signal_event) => {
        //         match signal_event {
        //             SignalEvent::BacktestConditionMatch(backtest_condition_match_event) => {
        //                 if backtest_condition_match_event.play_index == self.get_play_index().await {
        //                     self.create_order().await;
        //                 }
        //                 else {
        //                     tracing::warn!("{}: 当前k线缓存索引不匹配, 跳过", self.get_node_id());
        //                 }
        //             }
        //             _ => {}
        //         }
        //     }
        //     _ => {}
        // }
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {}

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) {
        match strategy_command {
            StrategyCommand::BacktestStrategy(BacktestNodeCommand::NodeReset(node_reset_params)) => {
                if self.get_node_id() == &node_reset_params.node_id {
                    let mut is_processing_order = self.is_processing_order.write().await;
                    is_processing_order.clear();
                    // 重置unfilled_virtual_order
                    let mut unfilled_virtual_order = self.unfilled_virtual_order.write().await;
                    unfilled_virtual_order.clear();
                    // 重置virtual_order_history
                    let mut virtual_order_history = self.virtual_order_history.write().await;
                    virtual_order_history.clear();

                    let response = NodeResetResponse::success(self.get_node_id().clone());
                    node_reset_params.responder.send(response.into()).unwrap();
                }
            }
            _ => {}
        }
    }
}
