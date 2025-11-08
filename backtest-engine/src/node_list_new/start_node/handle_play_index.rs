use super::StartNode;
use strategy_core::node::node_trait::NodeContextAccessor;
use strategy_core::node::context_trait::{NodeTaskControlExt, NodeIdentityExt};



impl StartNode {

    pub async fn listen_play_index_change(&self) {


        let (mut play_index_watch_rx, cancel_token, node_id) = self.with_ctx_read(|ctx| {
            let play_index_watch_rx = ctx.play_index_watch_rx().clone();
            let cancel_token = ctx.cancel_token().clone();
            let node_id = ctx.node_id().to_string();
            (play_index_watch_rx, cancel_token, node_id)
            
        }).await;

        let start_node = self.clone();

        // 节点接收播放索引变化
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 播放索引监听任务已中止", node_id);
                        break;
                    }
                    // 监听播放索引变化
                    receive_result = play_index_watch_rx.changed() => {
                        match receive_result {
                            Ok(_) => {
                                start_node.with_ctx_write_async(|ctx| {
                                    Box::pin(async move {
                                        ctx.send_play_signal().await;
                                    })
                                }).await;
                            }
                            Err(e) => {
                                tracing::error!("节点{}监听播放索引错误: {}", node_id, e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}