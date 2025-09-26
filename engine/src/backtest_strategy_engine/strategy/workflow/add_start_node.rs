use super::{BacktestNodeTrait, BacktestStrategyContext, BacktestStrategyFunction, StartNode};
use event_center::communication::backtest_strategy::{BacktestNodeCommand, StrategyCommandSender};
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::start_node_error::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

impl BacktestStrategyFunction {
    pub async fn add_start_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
    ) -> Result<(), StartNodeError> {
        let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);

        let (heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            let heartbeat = strategy_context_guard.heartbeat.clone();
            let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
            let strategy_stats = strategy_context_guard.strategy_stats.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx)
        };

        let mut node = StartNode::new(
            node_config,
            heartbeat,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            virtual_trading_system,
            strategy_stats,
            play_index_watch_rx,
        )?;
        let node_id = node.get_node_id().await;

        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;
        strategy_context_guard
            .add_node_command_sender(node_id.to_string(), node_command_tx)
            .await;

        let node = Box::new(node);

        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard.node_indices.insert(node_id.to_string(), node_index);

        Ok(())
    }
}
