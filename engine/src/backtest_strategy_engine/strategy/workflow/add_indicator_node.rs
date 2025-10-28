use super::{
    BacktestNodeTrait, BacktestStrategyContext, BacktestStrategyFunction,
    IndicatorNode, IndicatorNodeContext, BacktestNodeContextTrait, BacktestNodeContextAccessor,
    BacktestStrategyNodeError
};
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
    ) -> Result<(), BacktestStrategyNodeError> {
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

        let (node_id, node_name, indicator_keys, node_type) = node.with_ctx_read::<IndicatorNodeContext, _>(|ctx| {
            let node_id = ctx.get_node_id().clone();
            let node_name = ctx.get_node_name().clone();
            let indicator_keys_map = ctx.get_indicator_keys_ref();
            let indicator_keys = indicator_keys_map.keys().cloned().collect::<Vec::<IndicatorKey>>();
            let node_type = ctx.get_node_type().to_string();
            (node_id, node_name, indicator_keys, node_type)
        }).await?;

        let mut strategy_keys_guard = strategy_keys.write().await;
        strategy_keys_guard.extend(indicator_keys.iter().map(|key| (key.clone().into(), node_id.clone())));
        drop(strategy_keys_guard); // 显式释放写锁

        // set default output handle

        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;

        strategy_context_guard
            .add_node_command_sender(node_id.to_string(), node_command_tx)
            .await;
        // 添加节点benchmark
        strategy_context_guard.add_node_benchmark(node_id.clone(), node_name, node_type).await;

        let node = Box::new(node);

        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard.node_indices.insert(node_id.to_string(), node_index);

        Ok(())
    }
}
