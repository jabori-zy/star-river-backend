use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::IfElseNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use std::sync::Arc;
use snafu::ResultExt;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use types::error::engine_error::strategy_engine_error::node_error::if_else_node_error::*;

impl BacktestStrategyFunction {
    pub async fn add_if_else_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), IfElseNodeError> {
        tracing::info!("start to add if else node.");
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
            
        let (play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            // let event_publisher = strategy_context_guard.event_publisher.clone();
            // let command_publisher = strategy_context_guard.command_publisher.clone();
            // let command_receiver = strategy_context_guard.command_receiver.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (play_index_watch_rx)
        };

        let mut node = IfElseNode::new(
            node_config, 
            // event_publisher,
            // command_publisher,
            // command_receiver,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            strategy_inner_event_receiver,
            play_index_watch_rx,
        )?;
        let node_id = node.get_node_id().await;
        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;

        let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
        strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;

        let node = Box::new(node);
        
        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard.node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }



    


}
