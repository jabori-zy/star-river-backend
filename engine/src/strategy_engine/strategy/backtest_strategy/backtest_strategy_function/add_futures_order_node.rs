use crate::strategy_engine::node::BacktestNodeTrait;
use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::futures_order_node::futures_order_node_types::*;
use crate::strategy_engine::node::backtest_strategy_node::futures_order_node::FuturesOrderNode;
use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventReceiver;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;



impl BacktestStrategyFunction {
    pub async fn add_futures_order_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
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


        let strategy_command_rx = {
            let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
            let strategy_context_guard = context.read().await;
            let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
            strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;
            strategy_command_rx
        };
        
        let (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, virtual_trading_system_event_receiver, database, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            let event_publisher = strategy_context_guard.event_publisher.clone();
            let command_publisher = strategy_context_guard.command_publisher.clone();
            let command_receiver = strategy_context_guard.command_receiver.clone();
            let heartbeat = strategy_context_guard.heartbeat.clone();
            let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
            let virtual_trading_system_event_receiver = strategy_context_guard.virtual_trading_system.lock().await.get_virtual_trading_system_event_receiver();
            let database = strategy_context_guard.database.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, virtual_trading_system_event_receiver, database, play_index_watch_rx)
        };

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
        let mut context_guard = context.write().await;
        let node_index = context_guard.graph.add_node(node);
        context_guard.node_indices.insert(node_id, node_index);
        Ok(())
    }
}