use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::IfElseNode;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::if_else_node_type::*;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::condition::Case;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;

impl BacktestStrategyFunction {
    pub async fn add_if_else_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {
        
        let node_data = node_config["data"].clone();
        let node_id = node_config["id"].as_str().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap_or_default();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let backtest_config_json = node_data["backtestConfig"].clone();
        if backtest_config_json.is_null() {
            return Err("backtestConfig is null".to_string());
        }

        let cases_json = backtest_config_json["cases"].clone();
        let cases: Vec<Case> = serde_json::from_value(cases_json).unwrap();
        let if_else_node_backtest_config = IfElseNodeBacktestConfig {
            cases: cases.clone(),
        };

        let strategy_command_rx = {
            let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
            let strategy_context_guard = context.read().await;
            let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
            strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;
            strategy_command_rx
        };
        
        let (event_publisher, command_publisher, command_receiver, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            let event_publisher = strategy_context_guard.event_publisher.clone();
            let command_publisher = strategy_context_guard.command_publisher.clone();
            let command_receiver = strategy_context_guard.command_receiver.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (event_publisher, command_publisher, command_receiver, play_index_watch_rx)
        };

        let mut node = IfElseNode::new(
            strategy_id as i32, 
            node_id.to_string(),
            node_name.to_string(), 
            if_else_node_backtest_config, 
            event_publisher,
            command_publisher,
            command_receiver,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        node.set_output_handle().await;
        let node = Box::new(node);
        let mut context_guard = context.write().await;
        let node_index = context_guard.graph.add_node(node);
        context_guard.node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }

}
