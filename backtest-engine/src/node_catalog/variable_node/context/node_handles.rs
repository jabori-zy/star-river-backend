use star_river_core::order::OrderType;
use strategy_core::node::context_trait::{NodeHandleExt, NodeIdentityExt};
use tokio::sync::broadcast;

use super::VariableNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for VariableNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let variable_configs = self.node_config.variable_configs.clone();

        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!("[{node_name}] setting strategy output handle: {}", strategy_output_handle_id);
        self.add_output_handle(strategy_output_handle_id, tx);

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!("[{node_name}] setting default output handle: {}", default_output_handle_id);
        self.add_output_handle(default_output_handle_id, tx);

        for variable in variable_configs {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let output_handle_id = format!("{}_output_{}", node_id, variable.config_id());
            tracing::debug!("[{node_name}] setting variable output handle: {}", output_handle_id);
            self.add_output_handle(output_handle_id, tx);
        }
    }
}
