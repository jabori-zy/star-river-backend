use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeInfoExt},
    utils::generate_default_output_handle,
};
use tokio::sync::broadcast;

use super::IndicatorNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for IndicatorNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let selected_indicators = self.node_config.exchange_mode_config.as_ref().unwrap().selected_indicators.clone();

        // 添加默认出口
        let default_output_handle = generate_default_output_handle::<Self::NodeEvent>(&node_id);
        self.add_default_output_handle(default_output_handle);

        // 添加每一个indicator的出口
        for indicator in selected_indicators.iter() {
            let indicator_output_handle_id = indicator.output_handle_id.clone();
            let config_id = indicator.config_id;
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(false, config_id, indicator_output_handle_id, tx);
        }
    }
}
