use futures::StreamExt;
use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeIdentityExt, NodeTaskControlExt},
    node_trait::NodeContextAccessor,
};
use tokio_stream::wrappers::BroadcastStream;

use super::FuturesOrderNode;

impl FuturesOrderNode {
    // specific for futures order node
    pub async fn listen_source_node_events_spec(&self) {
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
            return;
        }
        // 为每个接收器独立创建监听任务
        for input_handle in input_handles {
            let context = self.context.clone();
            let cancel_token = cancel_token.clone();
            let node_name = node_name.clone();
            let input_handle_id = input_handle.input_handle_id.clone();

            // 为每个接收器创建独立的监听流
            let mut stream = BroadcastStream::new(input_handle.receiver());

            let context = context.clone();
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
                                    if let Err(e) = context_guard.handle_node_event_for_specific_order(
                                        node_event,
                                        &input_handle_id
                                    ).await {
                                        tracing::error!("@[{}] handle specific order event error: {}", node_name, e);
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
    }
}
