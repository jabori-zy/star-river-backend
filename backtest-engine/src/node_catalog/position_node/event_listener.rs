use futures::StreamExt;
use star_river_core::error::StarRiverErrorTrait;
use strategy_core::{
    event::node_common_event::{CommonEvent, NodeRunningLogEvent},
    node::{
        context_trait::{NodeCommunicationExt, NodeInfoExt, NodeTaskControlExt},
        node_trait::NodeContextAccessor,
    },
};
use tokio_stream::wrappers::BroadcastStream;

use super::PositionNode;

impl PositionNode {

    pub(super) async fn listen_vts_events(&self) {
        let (vts_event_receiver, cancel_token, node_name) = self
            .with_ctx_read(|ctx| {
                let receiver = ctx.vts_event_receiver.resubscribe();
                let cancel_token = ctx.cancel_token().clone();
                let node_name = ctx.node_name().clone();
                (receiver, cancel_token, node_name)
            })
            .await;

        // Create a stream for receiving VTS events
        let mut stream = BroadcastStream::new(vts_event_receiver);
        let context = self.context().clone();

        // Spawn task to receive VTS events
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // If cancel signal is triggered, abort task
                    _ = cancel_token.cancelled() => {
                        tracing::info!("[{}] virtual trading system events listener stopped", node_name);
                        break;
                    }
                    // Receive events
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                if let Err(e) = context_guard.handle_vts_event(event).await {
                                    let current_time = context_guard.strategy_time();
                                    let running_error_log: CommonEvent = NodeRunningLogEvent::error_with_time(
                                        context_guard.cycle_id().clone(),
                                        context_guard.strategy_id().clone(),
                                        context_guard.node_id().clone(),
                                        context_guard.node_name().clone(),
                                        &e,
                                        current_time,
                                    ).into();
                                    if let Err(e) = context_guard.strategy_bound_handle_send(running_error_log.into()) {
                                        e.report();
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("[{}] failed to receive VTS event: {}", node_name, e);
                            }
                            None => {
                                tracing::warn!("[{}] VTS event stream closed", node_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
