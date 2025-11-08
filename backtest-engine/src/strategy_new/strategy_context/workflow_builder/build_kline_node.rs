// std
use std::sync::Arc;

// third-party
use tokio::sync::{Mutex, mpsc};

use crate::node_command::BacktestNodeCommand;
// current crate
use super::BacktestStrategyContext;
use crate::{
    error::node_error::BacktestNodeError,
    node_list_new::kline_node::KlineNode,
};
use strategy_core::strategy::context_trait::StrategyCommunicationExt;


impl BacktestStrategyContext {

    pub async fn build_kline_node(
        &self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
    ) -> Result<KlineNode, BacktestNodeError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let play_index_watch_rx = self.play_index_watch_rx();
        
        let node = KlineNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            play_index_watch_rx,
        )?;
        Ok(node)
    }
}