use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::{IndicatorNodeEvent, KlineNodeEvent, VariableNodeEvent};
use strategy_core::{
    event::node_common_event::{CommonEvent, TriggerEvent, TriggerPayload},
    node::context_trait::{NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeIdentityExt, NodeRelationExt},
};

use super::IfElseNodeContext;
use crate::node::{
    node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
    node_event::BacktestNodeEvent,
};

#[async_trait]
impl NodeEventHandlerExt for IfElseNodeContext {
    type EngineEvent = Event;

    async fn handle_node_command(&mut self, node_command: Self::NodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload;
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::debug!("{}: 收到节点事件: {:?}", self.get_node_id(), node_event);

        // 检查是否需要更新接收事件(play_index相同)
        let should_update = match &node_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                self.play_index() == indicator_update_event.play_index
            }
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                self.play_index() == kline_update_event.play_index
            }
            BacktestNodeEvent::VariableNode(variable_node_event) => match variable_node_event {
                VariableNodeEvent::SysVarUpdate(sys_variable_updated_event) => self.play_index() == sys_variable_updated_event.play_index,
                VariableNodeEvent::CustomVarUpdate(custom_variable_update_event) => {
                    self.play_index() == custom_variable_update_event.play_index
                }
            },
            _ => false,
        };

        if should_update {
            self.update_received_event(node_event.clone());
        }

        match &node_event {
            BacktestNodeEvent::Common(signal_event) => match signal_event {
                CommonEvent::Trigger(_) => {
                    tracing::debug!("{}: 接收到trigger事件。 不需要逻辑判断", self.node_id());

                    self.handle_trigger_event().await;
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }
}

impl IfElseNodeContext {
    pub(super) fn update_received_event(&mut self, received_event: BacktestNodeEvent) {
        // tracing::debug!("接收到的变量消息: {:?}", received_event);
        let (from_node_id, from_variable_id) = match &received_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                (indicator_update_event.from_node_id().clone(), indicator_update_event.config_id)
            }
            BacktestNodeEvent::VariableNode(variable_node_event) => match variable_node_event {
                VariableNodeEvent::SysVarUpdate(sys_variable_updated_event) => (
                    sys_variable_updated_event.from_node_id().clone(),
                    sys_variable_updated_event.variable_config_id,
                ),
                VariableNodeEvent::CustomVarUpdate(custom_variable_updated_event) => (
                    custom_variable_updated_event.from_node_id().clone(),
                    custom_variable_updated_event.variable_config_id,
                ),
            },
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
            self.send_execute_over_event(self.play_index() as u64, None).unwrap();
            return;
        }

        let all_output_handles = self.output_handles();
        for (handle_id, handle) in all_output_handles.iter() {
            if handle_id == &format!("{}_strategy_output", self.node_id()) {
                continue;
            }

            if handle.is_connected() {
                let payload = TriggerPayload::new(self.play_index() as u64);
                let trigger_event: CommonEvent =
                    TriggerEvent::new(self.node_id().clone(), self.node_name().clone(), handle_id.clone(), payload).into();

                let _ = handle.send(trigger_event.into());
            }
        }
    }
}
