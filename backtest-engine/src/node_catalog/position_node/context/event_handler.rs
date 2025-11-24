use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::IfElseNodeEvent;
use strategy_core::{
    event::node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload},
    node::context_trait::{NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};

use super::PositionNodeContext;
use crate::{
    node::{
        node_command::{NodeResetRespPayload, NodeResetResponse},
        node_error::PositionNodeError,
    },
    node_catalog::position_node::{BacktestNodeCommand, context::BacktestNodeEvent},
};

#[async_trait]
impl NodeEventHandlerExt for PositionNodeContext {
    type EngineEvent = Event;
    type Error = PositionNodeError;

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), PositionNodeError> {
        Ok(())
    }

    async fn handle_source_node_event(&mut self, _node_event: Self::NodeEvent) -> Result<(), PositionNodeError> {
        Ok(())
    }

    async fn handle_command(&mut self, node_command: BacktestNodeCommand) -> Result<(), PositionNodeError> {
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
}

impl PositionNodeContext {
    pub async fn handle_node_event_for_independent_position_op(
        &mut self,
        node_event: BacktestNodeEvent,
        config_id: i32,
    ) -> Result<(), PositionNodeError> {
        match node_event {
            BacktestNodeEvent::Common(signal_event) => match signal_event {
                CommonEvent::Trigger(trigger_event) => {
                    tracing::debug!(
                        "{}config_id: {}: 收到触发事件，不获取仓位信息。节点是否是叶子节点: {:?}",
                        self.node_name(),
                        config_id,
                        trigger_event
                    );
                    if self.is_leaf_node() {
                        self.send_execute_over_event(self.play_index() as u64, None).unwrap();
                        let payload = ExecuteOverPayload::new(self.play_index() as u64, None);
                        let execute_over_event: CommonEvent =
                            ExecuteOverEvent::new(self.node_id().clone(), self.node_name().clone(), self.node_id().clone(), payload).into();
                        self.strategy_bound_handle().send(execute_over_event.into()).unwrap();
                    }
                    Ok(())
                }
                _ => Ok(()),
            },

            BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) => {
                tracing::debug!("{}: 收到订单事件: {:?}", self.node_name(), futures_order_node_event);
                if self.is_leaf_node() {
                    let payload = ExecuteOverPayload::new(self.play_index() as u64, None);
                    let execute_over_event: CommonEvent =
                        ExecuteOverEvent::new(self.node_id().clone(), self.node_name().clone(), self.node_id().clone(), payload).into();
                    self.strategy_bound_handle().send(execute_over_event.into()).unwrap();
                    Ok(())
                } else {
                    Ok(())
                }
            }
            BacktestNodeEvent::IfElseNode(ifelse_event) => match ifelse_event {
                IfElseNodeEvent::CaseFalse(_) | IfElseNodeEvent::ElseFalse(_) => {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(self.play_index() as u64, Some(config_id)).unwrap();
                        Ok(())
                    } else {
                        self.independent_position_op_send_trigger_event(config_id).await;
                        Ok(())
                    }
                }
                IfElseNodeEvent::CaseTrue(_) | IfElseNodeEvent::ElseTrue(_) => {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(self.play_index() as u64, Some(config_id)).unwrap();
                        Ok(())
                    } else {
                        self.independent_position_op_send_trigger_event(config_id).await;
                        Ok(())
                    }
                }
            },

            _ => Ok(()),
        }
    }
}
