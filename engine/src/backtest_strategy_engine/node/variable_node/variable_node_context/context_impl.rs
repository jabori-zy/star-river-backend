use super::{
    BacktestBaseNodeContext, BacktestNodeCommand, BacktestNodeContextTrait, BacktestNodeEvent, Command, DataFlow, Event, IfElseNodeEvent,
    NodeOutputHandle, NodeResetResponse, VariableNodeContext,
    context_utils::{filter_condition_trigger_configs, filter_dataflow_trigger_configs},
};
use async_trait::async_trait;
use event_center::event::node_event::{
    NodeEventTrait,
    backtest_node_event::{IndicatorNodeEvent, KlineNodeEvent},
};
use star_river_core::strategy::node_benchmark::CycleTracker;

use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for VariableNodeContext {
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
        tracing::info!("{}: 处理事件: {:?}", self.get_node_id(), event);
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        match node_event {
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(match_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.get_play_index() as u32);
                node_cycle_tracker.start_phase("handle_condition_trigger");
                // 过滤出condition trigger caseid相同的变量配置
                let condition_trigger_configs = filter_condition_trigger_configs(self.node_config.variable_configs.iter(), &match_event);
                self.handle_condition_trigger(&condition_trigger_configs).await;
                node_cycle_tracker.end_phase("handle_condition_trigger");
                let completed_tracker = node_cycle_tracker.end();
                self.add_node_cycle_tracker(self.get_node_id().clone(), completed_tracker).await;
            }
            // k线更新，处理dataflow
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.get_play_index() as u32);
                node_cycle_tracker.start_phase("handle_dataflow_trigger");
                // 过滤出dataflow trigger相同的变量配置
                let dataflow_trigger_configs = filter_dataflow_trigger_configs(
                    self.node_config.variable_configs.iter(),
                    kline_update_event.from_node_id(),
                    kline_update_event.config_id,
                );
                let dataflow = DataFlow::from(kline_update_event.kline.clone());
                self.handle_dataflow_trigger(&dataflow_trigger_configs, dataflow).await;
                node_cycle_tracker.end_phase("handle_dataflow_trigger");
                let completed_tracker = node_cycle_tracker.end();
                self.add_node_cycle_tracker(self.get_node_id().clone(), completed_tracker).await;
            }
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.get_play_index() as u32);
                node_cycle_tracker.start_phase("handle_dataflow_trigger");
                let dataflow_trigger_configs = filter_dataflow_trigger_configs(
                    self.node_config.variable_configs.iter(),
                    indicator_update_event.from_node_id(),
                    indicator_update_event.config_id,
                );
                let dataflow = DataFlow::from(indicator_update_event.indicator_value.clone());
                self.handle_dataflow_trigger(&dataflow_trigger_configs, dataflow).await;
                node_cycle_tracker.end_phase("handle_dataflow_trigger");
                let completed_tracker = node_cycle_tracker.end();
                self.add_node_cycle_tracker(self.get_node_id().clone(), completed_tracker).await;
            }
            _ => {}
        }
    }

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
