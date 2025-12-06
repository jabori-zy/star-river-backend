use std::sync::Arc;

use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use strategy_core::strategy::context_trait::{StrategyCommunicationExt, StrategyInfoExt};
use tokio::sync::{Mutex, broadcast, mpsc};
use virtual_trading::{command::VtsCommand, event::VtsEvent};

use super::BacktestStrategyContext;
use crate::{
    node::node_command::BacktestNodeCommand, node_catalog::position_node::PositionNode, strategy::strategy_error::BacktestStrategyError,
};

impl BacktestStrategyContext {
    pub async fn build_position_node(
        &mut self,
        node_config: serde_json::Value,
        node_command_receiver: mpsc::Receiver<BacktestNodeCommand>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        vts_command_sender: mpsc::Sender<VtsCommand>,
        vts_event_receiver: broadcast::Receiver<VtsEvent>,
    ) -> Result<PositionNode, BacktestStrategyError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let strategy_time_watch_rx = self.strategy_time_watch_rx();

        let node = PositionNode::new(
            self.cycle_watch_rx(),
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_receiver)),
            database,
            heartbeat,
            strategy_time_watch_rx,
            vts_command_sender,
            vts_event_receiver,
        )?;
        Ok(node)
    }
}
