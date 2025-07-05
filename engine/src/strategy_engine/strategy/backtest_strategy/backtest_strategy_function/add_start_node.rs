use super::BacktestStrategyFunction;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use types::strategy::{BacktestStrategyConfig, SimulatedConfig, TradeMode};
use types::strategy::node_command::NodeCommandSender;
use event_center::{CommandPublisher, CommandReceiver, EventPublisher, EventReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use heartbeat::Heartbeat;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use super::super::StrategyCommandPublisher;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::mpsc;

impl BacktestStrategyFunction {
    pub async fn add_start_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_publisher: &mut StrategyCommandPublisher,
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
        tracing::debug!("回测配置: {:?}", backtest_config_json);
        let backtest_config = serde_json::from_value::<BacktestStrategyConfig>(backtest_config_json).unwrap();
        

        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
        strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;
        
        
        
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
        );
        // 设置默认输出句柄
        node.set_output_handle().await;
        let node = Box::new(node);
        let node_index = graph.add_node(node);
        node_indices.insert(node_id.to_string(), node_index);
        Ok(())
    }
}
