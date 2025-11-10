use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeIdentityExt},
    utils::generate_default_output_handle_id,
};
use tokio::sync::broadcast;

use super::KlineNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for KlineNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let selected_symbols = self.node_config.exchange_mode_config.as_ref().unwrap().selected_symbols.clone();

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = generate_default_output_handle_id(&node_id);
        tracing::debug!("[{node_name}] set default output handle: {}", default_output_handle_id);

        self.add_output_handle(default_output_handle_id, tx);

        // 添加每一个symbol的出口
        for symbol in selected_symbols.iter() {
            let symbol_output_handle_id = symbol.output_handle_id.clone();
            tracing::debug!("[{node_name}] setting symbol output handle: {}", symbol_output_handle_id);
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(symbol_output_handle_id, tx);
        }
    }
}
