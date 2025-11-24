use std::sync::Arc;

use strategy_core::strategy::context_trait::{StrategyCommunicationExt, StrategyInfoExt};
use tokio::sync::{Mutex, mpsc};

use super::BacktestStrategyContext;
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::BacktestNodeError},
    node_catalog::variable_node::VariableNode,
    virtual_trading_system::BacktestVts,
};

impl BacktestStrategyContext {
    pub async fn build_variable_node(
        &mut self,
        node_config: serde_json::Value,
        node_command_rx: mpsc::Receiver<BacktestNodeCommand>,
        virtual_trading_system: Arc<Mutex<BacktestVts>>,
    ) -> Result<VariableNode, BacktestNodeError> {
        let strategy_command_sender = self.strategy_command_sender().clone();
        let current_time_watch_rx = self.current_time_watch_rx();
        let node = VariableNode::new(
            self.cycle_watch_rx(),
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            virtual_trading_system,
            current_time_watch_rx,
        )?;
        Ok(node)
    }
}
