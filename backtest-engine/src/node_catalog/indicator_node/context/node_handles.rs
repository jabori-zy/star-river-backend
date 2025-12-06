use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeInfoExt},
    utils::generate_default_output_handle,
};
use tokio::sync::broadcast;

use super::IndicatorNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for IndicatorNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let selected_indicators = self.node_config.exchange_mode()?.selected_indicators.clone();

        // Add default output handle
        let default_output_handle = generate_default_output_handle::<Self::NodeEvent>(&node_id, &node_name);
        self.add_default_output_handle(default_output_handle);

        // Add output handle for each indicator
        for indicator in selected_indicators.iter() {
            let indicator_output_handle_id = indicator.output_handle_id.clone();
            let config_id = indicator.config_id;
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(false, config_id, indicator_output_handle_id, tx);
        }
        Ok(())
    }
}
