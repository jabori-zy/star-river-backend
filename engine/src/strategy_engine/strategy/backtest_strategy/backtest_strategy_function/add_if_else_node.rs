use super::BacktestStrategyFunction;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::IfElseNode;
use event_center::{EventPublisher, CommandPublisher, CommandReceiver};
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::if_else_node_type::*;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::node::backtest_strategy_node::if_else_node::condition::Case;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use super::super::StrategyCommandPublisher;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::custom_type::PlayIndex;

impl BacktestStrategyFunction {
    pub async fn add_if_else_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>, 
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        node_command_sender: NodeCommandSender,
        strategy_command_publisher: &mut StrategyCommandPublisher,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
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
        tracing::debug!("cases_json: {:?}", cases_json);
        let cases: Vec<Case> = serde_json::from_value(cases_json).unwrap();
        let if_else_node_backtest_config = IfElseNodeBacktestConfig {
            cases: cases.clone(),
        };
        tracing::debug!("条件分支节点数据: {:?}", if_else_node_backtest_config);

        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
        strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;

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
        let node_index = graph.add_node(node);
        node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }

}
