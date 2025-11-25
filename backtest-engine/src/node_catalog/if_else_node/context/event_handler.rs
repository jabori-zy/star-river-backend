use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::{
    IfElseNodeEvent, IndicatorNodeEvent, KlineNodeEvent, VariableNodeEvent,
    if_else_node_event::{CaseFalseEvent, CaseFalsePayload, ElseFalseEvent, ElseFalsePayload},
};
use strategy_core::{
    event::node_common_event::CommonEvent,
    node::context_trait::{NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};

use super::IfElseNodeContext;
use crate::node::{
    node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
    node_error::IfElseNodeError,
    node_event::BacktestNodeEvent,
};

#[async_trait]
impl NodeEventHandlerExt for IfElseNodeContext {
    type EngineEvent = Event;

    async fn handle_command(&mut self, node_command: Self::NodeCommand) -> Result<(), Self::Error> {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload;
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    async fn handle_source_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), Self::Error> {
        if let BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(_))
        | BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(_))
        | BacktestNodeEvent::VariableNode(VariableNodeEvent::SysVarUpdate(_))
        | BacktestNodeEvent::VariableNode(VariableNodeEvent::CustomVarUpdate(_)) = node_event
        {
            self.update_received_event(node_event.clone());
            return Ok(());
        }

        if let BacktestNodeEvent::Common(signal_event) = node_event.clone() {
            if let CommonEvent::Trigger(_) = signal_event {
                self.handle_trigger_event().await;
                return Ok(());
            } else {
                return Ok(());
            }
        }

        if let BacktestNodeEvent::IfElseNode(ifelse_event) = node_event {
            match ifelse_event {
                IfElseNodeEvent::CaseTrue(_) | IfElseNodeEvent::ElseTrue(_) => {
                    self.set_superior_case_is_true(true);
                    return Ok(());
                }
                IfElseNodeEvent::CaseFalse(_) | IfElseNodeEvent::ElseFalse(_) => {
                    self.set_superior_case_is_true(false);
                    self.clear_received_event();
                    if self.is_leaf_node() {
                        let config_ids = self.output_handles().values().map(|handle| handle.config_id()).collect::<Vec<_>>();
                        for id in config_ids {
                            self.send_execute_over_event(Some(id), Some(self.strategy_time())).unwrap();
                        }
                    } else {
                        self.send_all_case_false_event();
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl IfElseNodeContext {
    pub(super) fn update_received_event(&mut self, received_event: BacktestNodeEvent) -> Result<(), IfElseNodeError> {
        // tracing::debug!("接收到的变量消息: {:?}", received_event);
        let (from_node_id, from_variable_id) = match &received_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                (indicator_update_event.node_id().clone(), indicator_update_event.config_id)
            }
            BacktestNodeEvent::VariableNode(variable_node_event) => match variable_node_event {
                VariableNodeEvent::SysVarUpdate(sys_variable_updated_event) => (
                    sys_variable_updated_event.node_id().clone(),
                    sys_variable_updated_event.variable_config_id,
                ),
                VariableNodeEvent::CustomVarUpdate(custom_variable_updated_event) => (
                    custom_variable_updated_event.node_id().clone(),
                    custom_variable_updated_event.variable_config_id,
                ),
            },
            BacktestNodeEvent::KlineNode(KlineNodeEvent::KlineUpdate(kline_update_event)) => {
                (kline_update_event.node_id().clone(), kline_update_event.config_id)
            }
            _ => return Ok(()),
        };

        self.received_message
            .entry((from_node_id.clone(), from_variable_id))
            .and_modify(|e| *e = Some(received_event.clone()))
            .or_insert(Some(received_event));
        // tracing::debug!("received_message: {:?}", self.received_message);

        self.update_received_flag(from_node_id, from_variable_id, true);
        Ok(())
    }

    fn clear_received_event(&mut self) {
        self.received_message.clear();
    }

    pub(super) async fn handle_trigger_event(&mut self) {
        if self.is_leaf_node() {
            let config_ids = self.output_handles().values().map(|handle| handle.config_id()).collect::<Vec<_>>();
            for id in config_ids {
                self.send_execute_over_event(Some(id), Some(self.strategy_time())).unwrap();
            }
        } else {
            self.send_all_case_false_event();
        }
    }

    fn send_all_case_false_event(&mut self) {
        let ids = self
            .output_handles()
            .values()
            .map(|handle| (handle.config_id(), handle.output_handle_id().clone()))
            .collect::<Vec<(i32, String)>>();
        for id in ids {
            let payload = CaseFalsePayload::new(id.0);
            let case_false_event: IfElseNodeEvent = CaseFalseEvent::new_with_time(
                self.cycle_id(),
                self.node_id().clone(),
                self.node_name().clone(),
                id.1.clone(),
                self.strategy_time(),
                payload,
            )
            .into();
            self.output_handle_send(case_false_event.into()).unwrap();
        }

        let payload = ElseFalsePayload;
        let else_false_event: IfElseNodeEvent = ElseFalseEvent::new_with_time(
            self.cycle_id(),
            self.node_id().clone(),
            self.node_name().clone(),
            self.default_output_handle().unwrap().output_handle_id().clone(),
            self.strategy_time(),
            payload,
        )
        .into();
        let default_output_handle = self.default_output_handle().unwrap();
        let _ = default_output_handle.send(else_false_event.into());
    }
}
