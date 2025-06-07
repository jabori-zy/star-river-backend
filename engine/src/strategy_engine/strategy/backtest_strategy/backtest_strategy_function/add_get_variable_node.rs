use super::BacktestStrategyFunction;
use crate::strategy_engine::node::BacktestNodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use crate::strategy_engine::node::backtest_strategy_node::get_variable_node::GetVariableNode;
use types::node::get_variable_node::*;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use virtual_trading::VirtualTradingSystem;


impl BacktestStrategyFunction {
    pub async fn add_get_variable_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        database: DatabaseConnection,
        strategy_command_sender: NodeCommandSender,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone();
        let node_id = node_config["id"].as_str().unwrap().to_string();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap().to_string();
        let backtest_config_json = node_data["backtestConfig"].clone();
        if backtest_config_json.is_null() {
            return Err("backtestConfig is null".to_string());
        }
        let backtest_config = serde_json::from_value::<GetVariableNodeBacktestConfig>(backtest_config_json).unwrap();
        let mut node = GetVariableNode::new(
            strategy_id as i32,
            node_id.clone(),
            node_name,
            backtest_config,
            event_publisher,
            command_publisher,
            command_receiver,
            response_event_receiver,
            heartbeat,
            database,
            strategy_command_sender,
            virtual_trading_system,
            strategy_inner_event_receiver,
        );
        node.set_output_handle().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
        Ok(())
    }
}