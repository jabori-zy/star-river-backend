use super::IfElseNodeContext;

use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use event_center::event::node_event::NodeEventTrait;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};
use event_center::event::node_event::backtest_node_event::indicator_node_event::IndicatorNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::variable_node_event::VariableNodeEvent;

impl IfElseNodeContext {
    pub(super) fn update_received_event(&mut self, received_event: BacktestNodeEvent) {
        // tracing::debug!("接收到的变量消息: {:?}", received_event);
        let (from_node_id, from_variable_id) = match &received_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => (
                indicator_update_event.from_node_id().clone(),
                indicator_update_event.config_id,
            ),
            BacktestNodeEvent::VariableNode(VariableNodeEvent::SysVariableUpdated(sys_variable_updated_event)) => (
                sys_variable_updated_event.from_node_id().clone(),
                sys_variable_updated_event.variable_config_id,
            ),
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                (kline_update_event.from_node_id().clone(), kline_update_event.config_id)
            }
            _ => return,
        };

        self.received_message
            .entry((from_node_id.clone(), from_variable_id))
            .and_modify(|e| *e = Some(received_event.clone()))
            .or_insert(Some(received_event));
        // tracing::debug!("received_message: {:?}", self.received_message);

        self.update_received_flag(from_node_id, from_variable_id, true);
    }

    pub(super) async fn handle_trigger_event(&mut self) {
        if self.is_leaf_node() {
            self.send_execute_over_event().await;
            return;
        }

        let all_output_handles = self.get_all_output_handles();
        for (handle_id, handle) in all_output_handles.iter() {
            if handle_id == &format!("{}_strategy_output", self.get_node_id()) {
                continue;
            }

            if handle.connect_count > 0 {
                let payload = TriggerPayload::new(self.get_play_index());
                let trigger_event: CommonEvent = TriggerEvent::new(
                    self.get_node_id().clone(),
                    self.get_node_name().clone(),
                    handle_id.clone(),
                    payload,
                )
                .into();

                let _ = handle.send(trigger_event.into());
            }
        }
    }
}
