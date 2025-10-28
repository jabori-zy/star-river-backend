use super::{
    BacktestNodeTrait, BacktestStrategyContext, BacktestStrategyFunction, 
    IfElseNode, IfElseNodeContext, BacktestStrategyNodeError, BacktestNodeContextTrait};
use event_center::communication::backtest_strategy::{BacktestNodeCommand, StrategyCommandSender};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use crate::backtest_strategy_engine::node::BacktestNodeContextAccessor;

impl BacktestStrategyFunction {
    pub async fn add_if_else_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
    ) -> Result<(), BacktestStrategyNodeError> {
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
        
        let (node_id, node_name, node_type) = node.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            let node_id = ctx.get_node_id().clone();
            let node_name = ctx.get_node_name().clone();
            let node_type = ctx.get_node_type().to_string();
            (node_id, node_name, node_type)
        }).await?;
        
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
