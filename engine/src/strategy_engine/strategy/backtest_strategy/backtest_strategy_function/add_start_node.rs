use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use types::strategy::node_command::NodeCommandSender;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use types::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::start_node_error::*;

impl BacktestStrategyFunction {
    pub async fn add_start_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), StartNodeError> {
        
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
        
        let (heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            // let event_publisher = strategy_context_guard.event_publisher.clone();
            // let command_publisher = strategy_context_guard.command_publisher.clone();
            // let command_receiver = strategy_context_guard.command_receiver.clone();
            let heartbeat = strategy_context_guard.heartbeat.clone();
            let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
            let strategy_stats = strategy_context_guard.strategy_stats.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx)
        };
        
        
        let mut node = StartNode::new(
            node_config,
            // event_publisher,
            // command_publisher,
            // command_receiver,
            heartbeat,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            strategy_inner_event_receiver,
            virtual_trading_system,
            strategy_stats,
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
