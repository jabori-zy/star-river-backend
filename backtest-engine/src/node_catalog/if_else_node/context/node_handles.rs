use async_trait::async_trait;
use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeIdentityExt},
    utils::generate_default_output_handle_id,
};
use tokio::sync::broadcast;

use super::IfElseNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for IfElseNodeContext {
    fn set_output_handles(&mut self) {
        // 添加else出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let else_output_handle_id = format!("{}_else_output", self.node_id()); // else分支作为默认出口
        tracing::debug!(
            "[{}] setting ELSE output handle: {}, as default output handle",
            self.node_name(),
            else_output_handle_id
        );
        self.add_output_handle(else_output_handle_id, tx);

        let cases = &self.node_config.cases;
        let case_output_handle_ids = cases.iter().map(|case| case.output_handle_id.clone()).collect::<Vec<String>>();

        case_output_handle_ids.into_iter().for_each(|id| {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{}] set case output handle: {}", self.node_name(), &id);
            self.add_output_handle(id, tx);
        });
    }
}
