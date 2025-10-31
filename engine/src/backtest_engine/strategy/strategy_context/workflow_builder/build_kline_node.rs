use super::BacktestStrategyContext;
use std::sync::Arc;
use tokio::sync::mpsc;
use event_center::communication::backtest_strategy::BacktestNodeCommand;
use star_river_core::error::node_error::backtest_node_error::BacktestNodeError;
use crate::backtest_engine::node::KlineNode;
use tokio::sync::Mutex;



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