use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use types::strategy::BacktestStrategyConfig;
use types::strategy::node_command::NodeCommandSender;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;

impl BacktestStrategyFunction {
    pub async fn add_start_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {

        let node_data = node_config["data"].clone();
        let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
        let node_id = node_config["id"].as_str().unwrap(); // 节点id
        let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
        // tracing::debug!("开始节点数据: {:?}, {:?}, {:?}", node_data["liveConfig"], node_data["backtestConfig"], node_data["simulatedConfig"]);
        // 解析策略配置
        let backtest_config_json = node_data["backtestConfig"].clone();
        if backtest_config_json.is_null() {
            return Err("backtestConfig is null".to_string());
        }
        // tracing::debug!("回测配置: {:?}", backtest_config_json);
        let backtest_config = serde_json::from_value::<BacktestStrategyConfig>(backtest_config_json).unwrap();
        

        let strategy_command_rx = {
            let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
            let strategy_context_guard = context.read().await;
            let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
            strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;
            strategy_command_rx
        };
        
        let (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            let event_publisher = strategy_context_guard.event_publisher.clone();
            let command_publisher = strategy_context_guard.command_publisher.clone();
            let command_receiver = strategy_context_guard.command_receiver.clone();
            let heartbeat = strategy_context_guard.heartbeat.clone();
            let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
            let strategy_stats = strategy_context_guard.strategy_stats.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, strategy_stats, play_index_watch_rx)
        };
        
        
        let mut node = StartNode::new(
            strategy_id as i32, 
            node_id.to_string(), 
            node_name.to_string(), 
            backtest_config, 
            event_publisher,
            command_publisher,
            command_receiver,
            heartbeat,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            strategy_inner_event_receiver,
            virtual_trading_system,
            strategy_stats,
            play_index_watch_rx,
        );
        // 设置默认输出句柄
        node.set_output_handle().await;
        let node = Box::new(node);

        let mut context_guard = context.write().await;
        let node_index = context_guard.graph.add_node(node);
        context_guard.node_indices.insert(node_id.to_string(), node_index);

        Ok(())
    }
}
