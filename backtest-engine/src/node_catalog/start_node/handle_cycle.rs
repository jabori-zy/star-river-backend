use std::sync::Arc;

use star_river_core::error::StarRiverErrorTrait;
use strategy_core::{
    event::node_common_event::{CommonEvent, NodeRunningLogEvent},
    node::{
        context_trait::{NodeCommunicationExt, NodeInfoExt, NodeTaskControlExt},
        node_trait::NodeContextAccessor,
    },
    strategy::cycle::Cycle,
};

use super::StartNode;

impl StartNode {
    pub async fn listen_cycle_id_change(&self) {
        let (mut cycle_id_watch_rx, cancel_token, node_name) = self
            .with_ctx_read(|ctx| {
                let cycle_id_watch_rx = ctx.cycle_watch_rx();
                let cancel_token = ctx.cancel_token().clone();
                let node_name = ctx.node_name().to_string();
                (cycle_id_watch_rx, cancel_token, node_name)
            })
            .await;

        let context = Arc::clone(self.context());

        tracing::info!("[{}]: start to listen play index change", node_name);
        // Node receives play index changes
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Abort task if cancel signal is triggered
                    _ = cancel_token.cancelled() => {
                        tracing::info!("[{}]: play index listen task stopped", node_name);
                        break;
                    }
                    // Listen for play index changes
                    receive_result = cycle_id_watch_rx.changed() => {
                        match receive_result {
                            Ok(_) => {
                                let context_guard = context.write().await;

                                let cycle = context_guard.cycle_watch_rx().borrow().clone();

                                match cycle {
                                    Cycle::Id(_) => {
                                        let result = context_guard.send_play_signal().await;

                                        if let Err(e) = result {
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
                                                e.report_log();
                                            };


                                        }
                                    }
                                    Cycle::Reset => {}
                                }



                            }
                            Err(e) => {
                                tracing::error!("[{}]: listen play index error: {}", node_name, e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
