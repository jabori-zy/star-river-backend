use std::sync::Arc;

use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use strategy_core::strategy::context_trait::StrategyCommunicationExt;
use tokio::sync::{Mutex, broadcast, mpsc};
use virtual_trading::{command::VtsCommand, event::VtsEvent};

use super::BacktestStrategyContext;
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError},
    node_catalog::futures_order_node::FuturesOrderNode,
};

impl BacktestStrategyContext {
    pub async fn build_futures_order_node(
        &mut self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        vts_command_sender: mpsc::Sender<VtsCommand>,
        vts_event_receiver: broadcast::Receiver<VtsEvent>,
    ) -> Result<FuturesOrderNode, BacktestNodeError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let play_index_watch_rx = self.play_index_watch_rx();

        let node = FuturesOrderNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            play_index_watch_rx,
            database,
            heartbeat,
            vts_command_sender,
            vts_event_receiver,
        )?;
        Ok(node)
    }
}
