use strategy_core::{
    error::NodeError,
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt},
        node_handles::NodeOutputHandle,
    },
};
use tokio::sync::broadcast;

use super::IfElseNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for IfElseNodeContext {
    fn set_output_handles(&mut self) -> Result<(), Self::Error> {
        // 添加else出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let else_output_handle_id = format!("{}_else_output", self.node_id()); // else分支作为默认出口
        tracing::debug!(
            "[{}] setting ELSE output handle: {}, as default output handle",
            self.node_name(),
            else_output_handle_id
        );
        self.add_output_handle(true, -2, else_output_handle_id, tx);

        let cases = &self.node_config.cases;
        let case_info = cases
            .iter()
            .map(|case| (case.case_id, case.output_handle_id.clone()))
            .collect::<Vec<(i32, String)>>();

        case_info.into_iter().for_each(|(case_id, output_handle_id)| {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{}] set case output handle: {}", self.node_name(), &output_handle_id);
            self.add_output_handle(false, case_id, output_handle_id, tx);
        });
        Ok(())
    }

    fn default_output_handle(&self) -> Result<&NodeOutputHandle<BacktestNodeEvent>, NodeError> {
        let default_handle_id = format!("{}_else_output", self.node_id());
        self.output_handle(&default_handle_id)
    }
}
