use std::sync::Arc;

use async_trait::async_trait;
use event_center::EventCenterSingleton;
use futures::{StreamExt, stream::select_all};
use star_river_core::error::StarRiverErrorTrait;
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;
use strategy_core::{
    event::strategy_event::StrategyRunningLogEvent,
    strategy::{
        context_trait::{
            StrategyCommunicationExt, StrategyEventHandlerExt, StrategyIdentityExt, StrategyInfoExt, StrategyTaskControlExt,
            StrategyWorkflowExt,
        },
        strategy_trait::{StrategyContextAccessor, StrategyEventListener},
    },
};
use strategy_stats::strategy_stats::{StrategyStatsAccessor, StrategyStatsCommunicationExt};
use tokio_stream::wrappers::BroadcastStream;
use virtual_trading::vts_trait::VtsCtxAccessor;

use super::BacktestStrategy;

#[async_trait]
impl StrategyEventListener for BacktestStrategy {
    async fn listen_node_events(&self) {
        let (receivers, cancel_token, strategy_name) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let mut nodes = ctx.topological_sort().unwrap();
                    let mut receivers = Vec::new();
                    let strategy_name = ctx.strategy_name();
                    for node in nodes.iter_mut() {
                        let receiver = node.subscribe_strategy_output_handle(strategy_name.clone()).await;
                        receivers.push(receiver);
                    }

                    let cancel_token = ctx.cancel_token().clone();
                    (receivers, cancel_token, strategy_name.clone())
                })
            })
            .await;

        if receivers.is_empty() {
            tracing::warn!("{}: no message receivers", strategy_name);
            return;
        }

        // Create a stream to receive events from nodes
        let streams: Vec<_> = receivers.into_iter().map(|receiver| BroadcastStream::new(receiver)).collect();

        let mut combined_stream = select_all(streams);

        let context_clone = self.context.clone();

        // Node receives data
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Abort task if cancel signal is triggered
                    _ = cancel_token.cancelled() => {
                        tracing::info!("#[{}] node event listener stopped", strategy_name);
                        break;
                    }
                    // Receive messages
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut state_guard = context_clone.write().await;
                                let result = state_guard.handle_node_event(event).await;
                                if let Err(e) = result {
                                    e.report_log();
                                    let current_time = state_guard.strategy_time();
                                    let running_error_log: BacktestStrategyEvent = StrategyRunningLogEvent::error_with_time(state_guard.cycle_id().clone(), state_guard.strategy_id().clone(), &e, current_time).into();
                                    if let Err(e) = EventCenterSingleton::publish(running_error_log.into()).await {
                                        e.report_log();
                                    };
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("#[{}] receive node event error: {}", strategy_name, e);
                            }
                            None => {
                                tracing::warn!("#[{}] all node event streams are closed", strategy_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn listen_strategy_command(&self) {
        let (strategy_name, command_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let strategy_name = ctx.strategy_name();
                    let command_receiver = ctx.strategy_command_receiver().clone();
                    (strategy_name.clone(), command_receiver)
                })
            })
            .await;
        tracing::info!("{}: strategy command listener started", strategy_name);

        let context = self.context.clone();
        tokio::spawn(async move {
            loop {
                // First get the command and immediately release the lock
                let command = {
                    let mut command_receiver_guard = command_receiver.lock().await;
                    let received_command = command_receiver_guard.recv().await;
                    if let Some(cmd) = received_command {
                        cmd
                    } else {
                        continue;
                    }
                };
                // Then acquire context write lock to handle the command
                let mut context_guard = context.write().await;
                context_guard.handle_strategy_command(command).await;
            }
        });
    }
}

impl BacktestStrategy {
    pub async fn listen_vts_events(&self) {
        let (strategy_name, cancel_token, vts_event_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let strategy_name = ctx.strategy_name();

                    let cancel_token = ctx.cancel_token().clone();
                    let vts = ctx.vts.with_ctx_read(|vts_ctx| vts_ctx.vts_event_receiver()).await;

                    (strategy_name.clone(), cancel_token, vts)
                })
            })
            .await;

        tracing::info!("{}: strategy vts event listener started", strategy_name);
        let mut stream = BroadcastStream::new(vts_event_receiver);

        let context = self.context.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("#[{}] vts event listener stopped", strategy_name);
                        break;
                    }
                    event = stream.next() => {
                        if let Some(Ok(event)) = event {
                            let mut context_guard = context.write().await;
                            let result = context_guard.handle_vts_event(event).await;
                            if let Err(e) = result {
                                e.report_log();
                                let current_time = context_guard.strategy_time();
                                let running_error_log: BacktestStrategyEvent = StrategyRunningLogEvent::error_with_time(context_guard.cycle_id().clone(), context_guard.strategy_id().clone(), &e, current_time).into();
                                if let Err(e) = EventCenterSingleton::publish(running_error_log.into()).await {
                                    e.report_log();
                                };
                            }
                        } else {
                            tracing::warn!("#[{}] strategy vts event listener closed", strategy_name);
                            break;
                        }
                    }
                }
            }
        });
    }

    pub async fn listen_strategy_stats_events(&self) {
        let (strategy_name, cancel_token, strategy_stats_event_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let strategy_name = ctx.strategy_name();
                    let cancel_token = ctx.cancel_token().clone();
                    let strategy_stats_event_receiver = ctx
                        .strategy_stats()
                        .with_ctx_read(|stats| stats.strategy_stats_event_receiver())
                        .await;
                    (strategy_name.clone(), cancel_token, strategy_stats_event_receiver)
                })
            })
            .await;

        let mut stream = BroadcastStream::new(strategy_stats_event_receiver);

        let context = Arc::clone(&self.context);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{}: strategy stats event listener task stopped", strategy_name);
                        break;
                    }
                    event = stream.next() => {
                        match event {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                context_guard.handle_strategy_stats_event(event).await.unwrap();
                            }
                        Some(Err(e)) => {
                            tracing::error!("{}: strategy stats event receive error: {}", strategy_name, e);
                        }
                        None => {
                            tracing::warn!("{}: strategy stats event stream closed", strategy_name);
                            break;
                        }
                        }
                    }
                }
            }
        });
    }
}
