use std::sync::Arc;

use strategy_core::strategy::context_trait::StrategyCommunicationExt;
use tokio::sync::{Mutex, mpsc};

use super::BacktestStrategyContext;
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError},
    node_catalog::indicator_node::IndicatorNode,
};

impl BacktestStrategyContext {
    pub async fn build_indicator_node(
        &mut self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
    ) -> Result<IndicatorNode, BacktestNodeError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let play_index_watch_rx = self.play_index_watch_rx();

        let node = IndicatorNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            play_index_watch_rx,
        )?;
        Ok(node)
    }
}
