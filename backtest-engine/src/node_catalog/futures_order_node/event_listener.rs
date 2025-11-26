use futures::StreamExt;
use star_river_core::error::StarRiverErrorTrait;
use strategy_core::{
    event::{
        node_common_event::CommonEvent,
        strategy_event::{StrategyRunningLogEvent, StrategyRunningLogSource},
    },
    node::{
        context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt, NodeTaskControlExt},
        node_trait::NodeContextAccessor,
    },
};
use tokio_stream::wrappers::BroadcastStream;

use crate::node::node_error::FuturesOrderNodeError;

use super::FuturesOrderNode;

impl FuturesOrderNode {
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
                                    tracing::error!("[{}] failed to handle virtual trading system event: {}", node_name, e);
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

    // specific for futures order node
    pub async fn listen_source_node_events_for_independent_order(&self) -> Result<(), FuturesOrderNodeError> {
        let (input_handles, cancel_token, node_name) = self
            .with_ctx_read(|ctx| {
                let input_handles = ctx.input_handles().to_vec();
                let cancel_token = ctx.cancel_token().clone();
                let node_name = ctx.node_name().to_string();
                (input_handles, cancel_token, node_name)
            })
            .await;

        if input_handles.is_empty() {
            tracing::warn!("@[{}] have no input handles", node_name);
            return Ok(());
        }

        // 为每个接收器独立创建监听任务
        for input_handle in input_handles {
            let context = self.context.clone();
            let cancel_token = cancel_token.clone();
            let node_name = node_name.clone();
            let input_handle_id = input_handle.input_handle_id.clone();
            let order_config_id = input_handle.config_id;

            // 为每个接收器创建独立的监听流
            let mut stream = BroadcastStream::new(input_handle.receiver());

            let context = context.clone();
            tracing::debug!("@[{}] start to listen source node events for order {}", node_name, order_config_id);
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        // 如果取消信号被触发，则中止任务
                        _ = cancel_token.cancelled() => {
                            tracing::info!("@[{}] input handle {} listener task cancelled", node_name, input_handle_id);
                            break;
                        }
                        // 接收消息
                        receive_result = stream.next() => {
                            match receive_result {
                                Some(Ok(node_event)) => {
                                    // 根据订单配置处理特定订单的事件
                                    let mut context_guard = context.write().await;
                                    let handle_result = context_guard.handle_node_event_for_independent_order(
                                        node_event,
                                        order_config_id
                                    ).await;
                                    if let Err(e) = handle_result {
                                        let current_time = context_guard.strategy_time();
                                        let running_error_log: CommonEvent = StrategyRunningLogEvent::error_with_time(
                                            context_guard.cycle_id().clone(),
                                            context_guard.strategy_id().clone(),
                                            context_guard.node_id().clone(),
                                            context_guard.node_name().clone(),
                                            StrategyRunningLogSource::Node,
                                            &e,
                                            current_time,
                                        ).into();
                                        if let Err(e) = context_guard.strategy_bound_handle_send(running_error_log.into()) {
                                            e.report();
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    tracing::error!("@[{}] input handle {} receive message error: {}", node_name, input_handle_id, e);
                                }
                                None => {
                                    tracing::warn!("@[{}] input handle {} message stream closed", node_name, input_handle_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }
        Ok(())
    }
}
