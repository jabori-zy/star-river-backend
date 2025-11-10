// std
use std::sync::Arc;

use strategy_core::strategy::context_trait::StrategyCommunicationExt;
// third-party
use tokio::sync::{Mutex, mpsc};

// current crate
use super::BacktestStrategyContext;
// current crate
use crate::node::node_command::BacktestNodeCommand;
use crate::{node::node_error::BacktestNodeError, node_catalog::start_node::StartNode};

impl BacktestStrategyContext {
    pub async fn build_start_node(
        &self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
    ) -> Result<StartNode, BacktestNodeError> {
        // let virtual_trading_system = self.virtual_trading_system().clone();
        // let strategy_stats = self.strategy_stats();
        let play_index_watch_rx = self.play_index_watch_rx();
        let strategy_command_sender = self.strategy_command_sender().clone();

        let node = StartNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            // virtual_trading_system,
            // strategy_stats,
            play_index_watch_rx,
        )?;
        Ok(node)
    }
}
