use super::{BacktestNodeTrait, BacktestStrategyContext, BacktestStrategyFunction, IfElseNode};
use event_center::communication::backtest_strategy::{BacktestNodeCommand, StrategyCommandSender};
use star_river_core::error::engine_error::strategy_engine_error::node_error::if_else_node_error::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

impl BacktestStrategyFunction {
    pub async fn add_if_else_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
    ) -> Result<(), IfElseNodeError> {
        tracing::info!("start to add if else node.");
        let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);

        let play_index_watch_rx = {
            let strategy_context_guard = context.read().await;
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            play_index_watch_rx
        };

        let mut node = IfElseNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            play_index_watch_rx,
        )?;
        let node_id = node.get_node_id().await;
        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;

        strategy_context_guard
            .add_node_command_sender(node_id.to_string(), node_command_tx)
            .await;
        // 添加节点benchmark
        strategy_context_guard.add_node_benchmark(node_id.clone(), node.get_node_name().await, node.get_node_type().await.to_string()).await;

        let node = Box::new(node);

        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard.node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }
}
