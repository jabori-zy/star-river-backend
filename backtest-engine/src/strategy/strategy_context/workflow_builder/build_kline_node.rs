// std
use std::sync::Arc;

use strategy_core::strategy::context_trait::{StrategyCommunicationExt, StrategyInfoExt};
// third-party
use tokio::sync::{Mutex, mpsc};

// current crate
use super::BacktestStrategyContext;
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError},
    node_catalog::kline_node::KlineNode,
};

impl BacktestStrategyContext {
    pub async fn build_kline_node(
        &self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
    ) -> Result<KlineNode, BacktestNodeError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let current_time_watch_rx = self.current_time_watch_rx();

        let node = KlineNode::new(
            self.cycle_watch_rx(),
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            current_time_watch_rx,
        )?;
        Ok(node)
    }
}
