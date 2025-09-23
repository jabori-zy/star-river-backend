use super::BacktestStrategyFunction;
use crate::strategy_engine::node::BacktestNodeTrait;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::strategy_engine::node::backtest_strategy_node::variable_node::VariableNode;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::mpsc;
use event_center::communication::strategy::{StrategyCommand, NodeCommandSender};
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::get_variable_node::*;

impl BacktestStrategyFunction {
    pub async fn add_variable_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        // response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), GetVariableNodeError> {
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);

        let (heartbeat, virtual_trading_system, database, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            // let event_publisher = strategy_context_guard.event_publisher.clone();
            // let command_publisher = strategy_context_guard.command_publisher.clone();
            // let command_receiver = strategy_context_guard.command_receiver.clone();
            let heartbeat = strategy_context_guard.heartbeat.clone();
            let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
            let database = strategy_context_guard.database.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (heartbeat, virtual_trading_system, database, play_index_watch_rx)
        };

        let mut node = VariableNode::new(
            node_config,
            // event_publisher,
            // command_publisher,
            // command_receiver,
            // response_event_receiver,
            heartbeat,
            database,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            virtual_trading_system,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        )?;
        // set output handle
        let node_id = node.get_node_id().await;
        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;
        let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
        strategy_command_publisher
            .add_sender(node_id.to_string(), strategy_command_tx)
            .await;

        let node = Box::new(node);

        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard
            .node_indices
            .insert(node_id.to_string(), node_index);
        Ok(())
    }
}
