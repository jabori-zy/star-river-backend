use super::BacktestStrategyFunction;
use crate::strategy_engine::node::BacktestNodeTrait;
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
use types::cache::CacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use virtual_trading::VirtualTradingSystem;
use super::super::StrategyCommandPublisher;

impl BacktestStrategyFunction {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>,
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<CacheKey>,
        node_config: Value,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_publisher: &mut StrategyCommandPublisher,
        node_command_sender: NodeCommandSender,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                Self::add_start_node(
                    graph, 
                    node_indices, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    heartbeat, 
                    node_command_sender, 
                    strategy_command_publisher, 
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
            }
            // 实时数据节点
            NodeType::KlineNode => {
                Self::add_kline_node(
                    graph, 
                    node_indices, 
                    cache_keys, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    market_event_receiver, 
                    response_event_receiver, 
                    heartbeat, 
                    node_command_sender,
                    strategy_command_publisher,
                    virtual_trading_system, 
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
                
            }
            // 指标节点
            NodeType::IndicatorNode => {
                Self::add_indicator_node(
                    graph, 
                    node_indices, 
                    cache_keys, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    response_event_receiver, 
                    node_command_sender, 
                    strategy_command_publisher,
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
                
            }
            
            // 条件分支节点
            NodeType::IfElseNode => {
                Self::add_if_else_node(
                    graph,
                    node_indices,
                    node_config,
                    event_publisher,
                    command_publisher,
                    command_receiver,
                    node_command_sender,
                    strategy_command_publisher,
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
            }
            // 订单节点
            NodeType::FuturesOrderNode => {
                Self::add_futures_order_node(
                    graph, 
                    node_indices, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    response_event_receiver, 
                    database, 
                    heartbeat, 
                    node_command_sender, 
                    strategy_command_publisher,
                    virtual_trading_system, strategy_inner_event_receiver).await.unwrap();
                Ok(())
            }
            // 持仓节点
            NodeType::PositionManagementNode => {
                Self::add_position_management_node(
                    graph, 
                    node_indices, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    response_event_receiver, 
                    database, 
                    heartbeat, 
                    node_command_sender, 
                    strategy_command_publisher, virtual_trading_system, strategy_inner_event_receiver).await.unwrap();
                Ok(())
                
            }
            // 获取变量节点
            NodeType::VariableNode => {
                Self::add_variable_node(
                    graph, 
                    node_indices, 
                    node_config, 
                    event_publisher, 
                    command_publisher, 
                    command_receiver, 
                    response_event_receiver, 
                    heartbeat, 
                    database, 
                    node_command_sender, 
                    strategy_command_publisher,
                    virtual_trading_system, 
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
                Err("不支持的节点类型".to_string())
            }
            
        }

    }



    
    
}
