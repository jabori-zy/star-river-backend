use super::KlineNodeContext;
use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;
use event_center::{
    communication::{
        engine::cache_engine::{CacheEngineCommand, ClearCacheCmdPayload, ClearCacheCommand},
        Response,
    },
    EventCenterSingleton,
};
use tokio::sync::oneshot;

impl KlineNodeContext {
    // 节点重置
    pub(super) async fn handle_node_reset(&self) {
        // 将缓存引擎中的，不在min_interval_symbols中的指标缓存键删除
        for (kline_key, _) in self.selected_symbol_keys.iter() {
            if !self.min_interval_symbols.contains(kline_key) {
                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = ClearCacheCmdPayload::new(
                    self.get_strategy_id().clone(),
                    kline_key.clone().into(),
                );
                let cmd: CacheEngineCommand = ClearCacheCommand::new(
                    self.get_node_id().clone(),
                    resp_tx,
                    Some(payload),
                ).into();
                
                let _ = EventCenterSingleton::send_command(cmd.into()).await;
                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    tracing::debug!("删除k线缓存成功");
                } else {
                    tracing::error!("删除k线缓存失败: {:#?}", response);
                }
            }
        }
    }
}
