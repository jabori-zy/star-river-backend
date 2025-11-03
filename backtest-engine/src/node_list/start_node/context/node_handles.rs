use super::{StartNodeAction, StartNodeContext};
use crate::node::node_context_trait::{NodeHandleTrait, NodeIdentity};
use async_trait::async_trait;
use tokio::sync::broadcast;
use event_center::event::node_event::BacktestNodeEvent;
use crate::node::node_utils::NodeUtils;

#[async_trait]
impl NodeHandleTrait<StartNodeAction> for StartNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = NodeUtils::generate_default_output_handle_id(&node_id);
        tracing::debug!("[{node_name}] setting default output handle: {}", default_output_handle_id);
        self.add_output_handle(default_output_handle_id, tx);
        
    }
}