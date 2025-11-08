use super::StartNodeContext;
use async_trait::async_trait;
use tokio::sync::broadcast;
use strategy_core::node::utils::generate_default_output_handle_id;

use strategy_core::node::context_trait::{NodeHandleExt, NodeIdentityExt};

#[async_trait]
impl NodeHandleExt for StartNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

        // 添加默认出口
        let (tx, _) = broadcast::channel::<Self::NodeEvent>(100);
        let default_output_handle_id = generate_default_output_handle_id(&node_id);
        tracing::debug!("[{node_name}] setting default output handle: {}", default_output_handle_id);
        self.add_output_handle(default_output_handle_id, tx);
        
    }
}