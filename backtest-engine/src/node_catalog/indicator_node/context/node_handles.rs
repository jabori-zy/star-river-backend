use super::IndicatorNodeContext;
use async_trait::async_trait;
use tokio::sync::broadcast;
use strategy_core::node::utils::generate_default_output_handle_id;
use crate::node::node_event::BacktestNodeEvent;
use strategy_core::node::context_trait::{NodeHandleExt, NodeIdentityExt};

impl NodeHandleExt for IndicatorNodeContext {
    fn set_output_handles(&mut self) {

        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let selected_indicators = self.node_config.exchange_mode_config.as_ref().unwrap().selected_indicators.clone();

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = generate_default_output_handle_id(&node_id);
        self.add_output_handle(default_output_handle_id, tx);

        // 添加每一个indicator的出口
        for indicator in selected_indicators.iter() {
            let indicator_output_handle_id = indicator.output_handle_id.clone();
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(indicator_output_handle_id, tx);
        }
    }
}