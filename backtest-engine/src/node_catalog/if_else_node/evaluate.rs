use strategy_core::node::{
    context_trait::{NodeIdentityExt, NodeTaskControlExt},
    node_trait::NodeContextAccessor,
};

use super::IfElseNode;
use crate::node::node_error::IfElseNodeError;

impl IfElseNode {
    pub async fn evaluate(&self) -> Result<(), IfElseNodeError> {
        let (node_id, cancel_token) = self
            .with_ctx_read(|ctx| {
                let node_id = ctx.node_id().clone();
                let cancel_token = ctx.cancel_token().clone();
                (node_id, cancel_token)
            })
            .await;

        let context = self.context().clone();
        tokio::spawn(async move {
            loop {
                if cancel_token.is_cancelled() {
                    tracing::info!("{} 节点条件判断进程已中止", node_id);
                    break;
                }

                let should_evaluate = {
                    let ctx_guard = context.read().await;
                    // if node is nested, and superior case is true, then should evaluate
                    ctx_guard.is_all_value_received() && (!ctx_guard.is_nested() || ctx_guard.superior_case_is_true())
                };
                // tracing::info!("[{}] should evaluate: {:?}", node_id, should_evaluate);

                if should_evaluate {
                    let mut ctx_guard = context.write().await;
                    if let Err(e) = ctx_guard.evaluate().await {
                        tracing::error!("[{}] Evaluation failed: {:?}", node_id, e);
                    }
                    ctx_guard.reset_received_flag();
                }
            }
        });

        Ok(())
    }
}
