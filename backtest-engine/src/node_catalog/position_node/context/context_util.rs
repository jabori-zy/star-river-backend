use strategy_core::node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeRelationExt};

use super::PositionNodeContext;

impl PositionNodeContext {
    pub(super) async fn independent_position_op_send_trigger_event(&self, config_id: i32) {
        let all_output_handles = self.output_handles();
        tracing::debug!("send trigger event to position output handles: {:#?}", all_output_handles);
        let futures = all_output_handles
            .values()
            .filter(|handle| handle.config_id() == config_id)
            .map(|handle| self.send_trigger_event(handle.output_handle_id()));

        futures::future::join_all(futures).await;
    }
}
