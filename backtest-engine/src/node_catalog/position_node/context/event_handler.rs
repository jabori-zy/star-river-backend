use event_center::Event;
use strategy_core::node::context_trait::{NodeEventHandlerExt, NodeIdentityExt, NodeHandleExt};
use super::PositionNodeContext;
use async_trait::async_trait;
use crate::node_catalog::position_node::BacktestNodeCommand;
use crate::node::node_command::{NodeResetRespPayload, NodeResetResponse};
use crate::node_catalog::position_node::context::BacktestNodeEvent;
use strategy_core::event::node_common_event::CommonEvent;
use strategy_core::node::context_trait::NodeRelationExt;
use strategy_core::event::node_common_event::ExecuteOverPayload;
use strategy_core::event::node_common_event::ExecuteOverEvent;



#[async_trait]
impl NodeEventHandlerExt for PositionNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }

    async fn handle_node_event(&mut self, node_event: Self::NodeEvent) {
        match node_event {
            BacktestNodeEvent::Common(signal_event) => match signal_event {
                CommonEvent::Trigger(_) => {
                    // tracing::debug!(
                    //     "{}: 收到触发事件，不获取仓位信息。节点是否是叶子节点: {}",
                    //     self.node_name(),
                    //     self.is_leaf_node()
                    // );
                    if self.is_leaf_node() {
                        let payload = ExecuteOverPayload::new(self.play_index() as u64);
                        let execute_over_event: CommonEvent = ExecuteOverEvent::new(
                            self.node_id().clone(),
                            self.node_name().clone(),
                            self.node_id().clone(),
                            payload,
                        )
                        .into();
                        self.strategy_bound_handle().send(execute_over_event.into()).unwrap();
                    }
                }
                _ => {}
            },

            BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) => {
                tracing::debug!("{}: 收到订单事件: {:?}", self.node_name(), futures_order_node_event);
                if self.is_leaf_node() {
                    let payload = ExecuteOverPayload::new(self.play_index() as u64);
                    let execute_over_event: CommonEvent = ExecuteOverEvent::new(
                        self.node_id().clone(),
                        self.node_name().clone(),
                        self.node_id().clone(),
                        payload,
                    )
                    .into();
                    self.strategy_bound_handle().send(execute_over_event.into()).unwrap();
                }
            }

            _ => {}
        }
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
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
}