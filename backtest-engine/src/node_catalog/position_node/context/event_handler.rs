use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::IfElseNodeEvent;
use strategy_core::{
    event::node_common_event::CommonEvent,
    node::context_trait::{NodeCommunicationExt, NodeEventHandlerExt, NodeInfoExt, NodeRelationExt},
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
                    let response = NodeResetResponse::success(self.node_id().clone(), self.node_name().clone(), payload);
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
        // tracing::debug!("{}: 收到事件: {:?}", self.node_name(), node_event);
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
                        self.send_execute_over_event(Some(config_id), Some("handle trigger event for position node".to_string()), Some(self.strategy_time()))?;
                    }
                    Ok(())
                }
                _ => Ok(()),
            },

            BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) => {
                tracing::debug!("{}: 收到订单事件: {:?}", self.node_name(), futures_order_node_event);
                if self.is_leaf_node() {
                    self.send_execute_over_event(Some(config_id), Some("handle futures order node event for position node".to_string()), Some(self.strategy_time()))?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            BacktestNodeEvent::IfElseNode(ifelse_event) => match ifelse_event {
                IfElseNodeEvent::CaseFalse(_) | IfElseNodeEvent::ElseFalse(_) => {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(Some(config_id), Some("handle case false event for position node".to_string()), Some(self.strategy_time()))?;
                        Ok(())
                    } else {
                        self.independent_position_op_send_trigger_event(config_id, Some("handle case false event for position node".to_string())).await?;
                        Ok(())
                    }
                }
                IfElseNodeEvent::CaseTrue(_) | IfElseNodeEvent::ElseTrue(_) => {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(Some(config_id), Some("handle case true event for position node".to_string()), Some(self.strategy_time()))?;
                        Ok(())
                    } else {
                        self.independent_position_op_send_trigger_event(config_id, Some("handle case true event for position node".to_string())).await?;
                        Ok(())
                    }
                }
            },

            _ => Ok(()),
        }
    }
}
