use strategy_core::node::{
    context_trait::{NodeIdentityExt, NodeTaskControlExt},
    node_trait::NodeContextAccessor,
};
use tokio::time::{Duration, interval};

use super::IfElseNode;
use crate::node::node_error::IfElseNodeError;

impl IfElseNode {
    pub async fn evaluate(&self) -> Result<(), IfElseNodeError> {
        let (node_id, cancel_token) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let node_id = ctx.node_id().clone();
                    let cancel_token = ctx.cancel_token().clone();
                    (node_id, cancel_token)
                })
            })
            .await;

        let node = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(10));

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点条件判断进程已中止", node_id);
                        break;
                    }
                    _ = interval.tick() => {
                        let should_evaluate = node.with_ctx_read(|ctx| {
                            ctx.is_all_value_received()
                        }).await;

                        if should_evaluate {
                            if let Err(e) = node.with_ctx_write_async(|ctx| {
                                Box::pin(async move {
                                    let result = ctx.evaluate().await;
                                    ctx.reset_received_flag();
                                    result
                                })
                            }).await {
                                tracing::error!("[{}] Evaluation failed: {:?}", node_id, e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
