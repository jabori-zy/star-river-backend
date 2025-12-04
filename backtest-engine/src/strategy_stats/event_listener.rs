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

        // 创建一个流，用于接收节点传递过来的message
        let mut stream = BroadcastStream::new(receiver);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("策略统计模块虚拟交易系统事件监听任务已中止");
                        break;
                    }
                    // 接收消息
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut guard = context.write().await;
                                if let Err(e) = guard.handle_vts_event(event).await {
                                    tracing::error!("处理虚拟交易系统事件失败: {}", e);
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("策略统计模块接收消息错误: {}", e);
                            }
                            None => {
                                tracing::warn!("策略统计模块所有消息流已关闭");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
