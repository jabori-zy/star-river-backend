use async_trait::async_trait;
use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeInfoExt},
    utils::generate_default_output_handle,
};

use super::StartNodeContext;

#[async_trait]
impl NodeHandleExt for StartNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        let node_id = self.node_id().clone();
        // 添加默认出口
        let default_output_handle = generate_default_output_handle::<Self::NodeEvent>(&node_id);
        self.add_default_output_handle(default_output_handle);
        Ok(())
    }
}
