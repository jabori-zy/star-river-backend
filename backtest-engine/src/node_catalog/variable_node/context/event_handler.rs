use super::VariableNodeContext;
use async_trait::async_trait;
use strategy_core::node::context_trait::NodeEventHandlerExt;
use crate::node::{node_command::NodeResetRespPayload, node_event::BacktestNodeEvent};
use crate::node::node_command::BacktestNodeCommand;
use event_center::Event;
use strategy_core::node::context_trait::NodeIdentityExt;
use star_river_event::backtest_strategy::node_event::IfElseNodeEvent;
use strategy_core::benchmark::node_benchmark::CycleTracker;
use star_river_event::backtest_strategy::node_event::{KlineNodeEvent, IndicatorNodeEvent};
use strategy_core::node_infra::variable_node::trigger::dataflow::DataFlow;
use strategy_core::node::context_trait::NodeBenchmarkExt;
use crate::node::node_command::NodeResetResponse;
use super::config_filter::{filter_condition_trigger_configs, filter_dataflow_trigger_configs};


#[async_trait]
impl NodeEventHandlerExt for VariableNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        match node_event {
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(match_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.play_index() as u32);
                node_cycle_tracker.start_phase("handle_condition_trigger");
                // 过滤出condition trigger caseid相同的变量配置
                let condition_trigger_configs = filter_condition_trigger_configs(self.node_config.variable_configs.iter(), &match_event);
                self.handle_condition_trigger(&condition_trigger_configs).await;
                node_cycle_tracker.end_phase("handle_condition_trigger");
                let completed_tracker = node_cycle_tracker.end();
                self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
            }
            // k线更新，处理dataflow
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.play_index() as u32);
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
                self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
            }
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                let mut node_cycle_tracker = CycleTracker::new(self.play_index() as u32);
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
                self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
            }
            _ => {}
        }
    }
    
    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let paylod = NodeResetRespPayload;
                    let response = NodeResetResponse::success(self.node_id().clone(), paylod);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}