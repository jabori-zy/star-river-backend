use super::LiveStrategyFunction;
use crate::strategy_engine::node::LiveNodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use event_center::{Event, EventPublisher};
use serde_json::Value;
use std::str::FromStr;
use crate::strategy_engine::node::node_types::NodeType;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use types::cache::Key;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;

impl LiveStrategyFunction {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn LiveNodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<Key>,
        node_config: Value,
        // event_publisher: EventPublisher, 
        // command_publisher: CommandPublisher,
        // command_receiver: Arc<Mutex<CommandReceiver>>,
        // market_event_receiver: EventReceiver,
        // response_event_receiver: EventReceiver,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: NodeCommandSender,
    ) -> Result<(), String> {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                Self::add_start_node(graph, node_indices, node_config, event_publisher, command_publisher, command_receiver, strategy_command_sender).await;
                Ok(())
            }
            // 实时数据节点
            NodeType::KlineNode => {
                Self::add_kline_node(graph, node_indices, cache_keys, node_config, event_publisher, command_publisher, command_receiver, market_event_receiver, response_event_receiver, heartbeat, strategy_command_sender).await;
                Ok(())
                
            }
            // 指标节点
            NodeType::IndicatorNode => {
                Self::add_indicator_node(graph, node_indices, cache_keys, node_config, event_publisher, command_publisher, command_receiver, response_event_receiver, strategy_command_sender).await;
                Ok(())
                
            }
            
            // 条件分支节点
            NodeType::IfElseNode => {
                Self::add_if_else_node(graph,node_indices,node_config,event_publisher,command_publisher,command_receiver, strategy_command_sender).await;
                Ok(())
            }
            // 订单节点
            NodeType::OrderNode => {
                Self::add_order_node(graph, node_indices, node_config, event_publisher, command_publisher, command_receiver, response_event_receiver, exchange_engine, database, heartbeat, strategy_command_sender).await;
                Ok(())
            }
            // 持仓节点
            NodeType::PositionNode => {
                Self::add_position_node(graph, node_indices, node_config, event_publisher, command_publisher, command_receiver, response_event_receiver, exchange_engine, database, heartbeat, strategy_command_sender).await;
                Ok(())
                
            }
            // 获取变量节点
            NodeType::GetVariableNode => {
                Self::add_get_variable_node(graph, node_indices, node_config, event_publisher, command_publisher, command_receiver, response_event_receiver, exchange_engine, heartbeat, database, strategy_command_sender).await;
                Ok(())
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
                Err("不支持的节点类型".to_string())
            }
            
        }

    }



    
    
}
