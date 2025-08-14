use crate::strategy_engine::node::BacktestNodeTrait;
use super::BacktestStrategyFunction;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use event_center::EventPublisher;
use crate::strategy_engine::node::backtest_strategy_node::futures_order_node::futures_order_node_types::*;
use crate::strategy_engine::node::backtest_strategy_node::futures_order_node::FuturesOrderNode;
use std::sync::Arc;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;
use virtual_trading::VirtualTradingSystem;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use super::super::StrategyCommandPublisher;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::virtual_trading_system::event::VirtualTradingSystemEventReceiver;
use types::custom_type::PlayIndex;

impl BacktestStrategyFunction {
    pub async fn add_futures_order_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_publisher: &mut StrategyCommandPublisher,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone(); // 节点数据

        let node_id = node_config["id"].as_str().unwrap().to_string(); // 节点id
        let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
        let node_name = node_data["nodeName"].as_str().unwrap().to_string(); // 节点名称
        let backtest_config_json = node_data["backtestConfig"].clone();
        if backtest_config_json.is_null() {
            return Err("backtestConfig is null".to_string());
        }
        let backtest_config = serde_json::from_value::<FuturesOrderNodeBacktestConfig>(backtest_config_json).unwrap();


        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
        strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;

        let mut node = FuturesOrderNode::new(
            strategy_id as i32,
            node_id.clone(),
            node_name,
            backtest_config,
            event_publisher,
            command_publisher,
            command_receiver,
            response_event_receiver,
            database,
            heartbeat,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            virtual_trading_system,
            strategy_inner_event_receiver,
            virtual_trading_system_event_receiver,
            play_index_watch_rx,
        );
        node.set_output_handle().await;

        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id, node_index);
        Ok(())
    }
}