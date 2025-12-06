use strategy_core::node::context_trait::{NodeHandleExt, NodeInfoExt};
use tokio::sync::broadcast;

use super::PositionNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for PositionNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let position_operations = self.node_config.position_operations.clone();

        // Add output for each order
        for position_operation in position_operations.iter() {
            let success_output_handle_id = format!(
                "{}_{}_success_output_{}",
                node_id,
                position_operation.position_operation.to_string(),
                position_operation.config_id
            );
            let failed_output_handle_id = format!(
                "{}_{}_failed_output_{}",
                node_id,
                position_operation.position_operation.to_string(),
                position_operation.config_id
            );
            let (success_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let (failed_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting success output handle: {}", success_output_handle_id);
            self.add_output_handle(false, position_operation.config_id, success_output_handle_id, success_tx);
            tracing::debug!("[{node_name}] setting failed output handle: {}", failed_output_handle_id);
            self.add_output_handle(false, position_operation.config_id, failed_output_handle_id, failed_tx);
        }
        Ok(())
    }
}
