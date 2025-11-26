use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeInfoExt},
    utils::generate_default_output_handle,
};
use tokio::sync::broadcast;

use super::VariableNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for VariableNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let variable_configs = self.node_config.variable_configs.clone();

        // 添加默认出口
        let default_output_handle = generate_default_output_handle::<Self::NodeEvent>(&node_id, &node_name);
        self.add_default_output_handle(default_output_handle);

        for variable in variable_configs {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let output_handle_id = variable.output_handle_id().clone();
            let config_id = variable.config_id();
            tracing::debug!("[{node_name}] setting variable output handle: {}", output_handle_id);
            self.add_output_handle(false, config_id, output_handle_id, tx);
        }
        Ok(())
    }
}
