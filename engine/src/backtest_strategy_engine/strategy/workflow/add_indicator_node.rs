use super::{BacktestNodeTrait, BacktestStrategyContext, BacktestStrategyFunction, IndicatorNode, IndicatorNodeContext};
use event_center::communication::backtest_strategy::{BacktestNodeCommand, StrategyCommandSender};
use star_river_core::error::engine_error::strategy_engine_error::node_error::indicator_node_error::*;
use star_river_core::key::key::IndicatorKey;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

impl BacktestStrategyFunction {
    pub async fn add_indicator_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
    ) -> Result<(), IndicatorNodeError> {
        let (node_command_tx, node_command_rx) = mpsc::channel::<BacktestNodeCommand>(100);

        let (strategy_keys, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            let strategy_keys = strategy_context_guard.keys.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (strategy_keys, play_index_watch_rx)
        };

        let mut node = IndicatorNode::new(
            node_config,
            strategy_command_sender,
            Arc::new(Mutex::new(node_command_rx)),
            play_index_watch_rx,
        )?;

        let indicator_keys: Vec<IndicatorKey> = {
            let node_ctx = node.get_context();
            let node_ctx_guard = node_ctx.read().await;
            let node_ctx_guard = node_ctx_guard.as_any().downcast_ref::<IndicatorNodeContext>().unwrap();
            let indicator_keys_map = node_ctx_guard.get_indicator_keys_ref().clone();
            indicator_keys_map.keys().cloned().collect()
        };

        let node_id = node.get_node_id().await;

        let mut strategy_keys_guard = strategy_keys.write().await;
        strategy_keys_guard.extend(indicator_keys.iter().map(|key| (key.clone().into(), node_id.clone())));
        drop(strategy_keys_guard); // 显式释放写锁

        // set default output handle

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
