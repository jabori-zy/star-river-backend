use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeInfoExt},
    utils::generate_default_output_handle,
};
use tokio::sync::broadcast;

use super::KlineNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for KlineNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let selected_symbols = self.node_config.exchange_mode()?.selected_symbols.clone();

        // 添加默认出口
        let default_output_handle = generate_default_output_handle::<Self::NodeEvent>(&node_id, &node_name);
        self.add_default_output_handle(default_output_handle);

        // 添加每一个symbol的出口
        for symbol in selected_symbols.iter() {
            let symbol_output_handle_id = symbol.output_handle_id.clone();
            let config_id = symbol.config_id;
            tracing::debug!("[{node_name}] setting symbol output handle: {}", symbol_output_handle_id);
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(false, config_id, symbol_output_handle_id, tx);
        }
        Ok(())
    }
}
