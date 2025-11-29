use star_river_core::error::StarRiverErrorTrait;
use strategy_core::{
    event::node_common_event::{CommonEvent, NodeRunningLogEvent},
    node::{
        context_trait::{NodeCommunicationExt, NodeInfoExt, NodeTaskControlExt},
        node_trait::NodeContextAccessor,
    },
};

use super::IfElseNode;
use crate::node::node_error::IfElseNodeError;

impl IfElseNode {
    pub async fn evaluate(&self) -> Result<(), IfElseNodeError> {
        let (node_name, cancel_token) = self
            .with_ctx_read(|ctx| {
                let node_name = ctx.node_name().clone();
                let cancel_token = ctx.cancel_token().clone();
                (node_name, cancel_token)
            })
            .await;

        let context = self.context().clone();
        tokio::spawn(async move {
            loop {
                if cancel_token.is_cancelled() {
                    tracing::info!("[{}] condition evaluation task cancelled", node_name);
                    break;
                }

                let should_evaluate = {
                    let ctx_guard = context.read().await;
                    // if node is nested, and superior case is true, then should evaluate
                    ctx_guard.is_all_value_received() && (!ctx_guard.is_nested() || ctx_guard.superior_case_status())
                };
                // tracing::info!("[{}] should evaluate: {:?}", node_name, should_evaluate);

                if should_evaluate {
                    let mut ctx_guard = context.write().await;
                    if let Err(e) = ctx_guard.evaluate().await {
                        let current_time = ctx_guard.strategy_time();
                        let running_error_log: CommonEvent = NodeRunningLogEvent::error_with_time(
                            ctx_guard.cycle_id().clone(),
                            ctx_guard.strategy_id().clone(),
                            ctx_guard.node_id().clone(),
                            ctx_guard.node_name().clone(),
                            &e,
                            current_time,
                        )
                        .into();
                        if let Err(e) = ctx_guard.strategy_bound_handle_send(running_error_log.into()) {
                            e.report();
                        }
                    }
                    ctx_guard.reset_received_flag();
                }
            }
        });

        Ok(())
    }
}
