use std::sync::Arc;

use futures::StreamExt;
use strategy_stats::strategy_stats::{StrategyStatsAccessor, StrategyStatsInfoExt};
use tokio_stream::wrappers::BroadcastStream;

use super::BacktestStrategyStats;

impl BacktestStrategyStats {
    pub async fn listen_vts_events(&self) {
        let (receiver, cancel_token) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let receiver = ctx.vts.context.read().await.vts_event_receiver();
                    let cancel_token = ctx.cancel_token().clone();
                    (receiver, cancel_token)
                })
            })
            .await;

        let context = Arc::clone(&self.context);

        // Create a stream to receive messages from nodes
        let mut stream = BroadcastStream::new(receiver);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Abort task if cancel signal is triggered
                    _ = cancel_token.cancelled() => {
                        tracing::info!("Strategy stats module virtual trading system event listener task aborted");
                        break;
                    }
                    // Receive messages
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut guard = context.write().await;
                                if let Err(e) = guard.handle_vts_event(event).await {
                                    tracing::error!("Failed to handle virtual trading system event: {}", e);
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("Strategy stats module message receiving error: {}", e);
                            }
                            None => {
                                tracing::warn!("Strategy stats module all message streams closed");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
