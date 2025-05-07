use super::LiveStrategyFunction;
use crate::strategy_engine::node::NodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::Indicators;
use event_center::{Event, EventPublisher};
use serde_json::Value;
use std::str::FromStr;
use crate::strategy_engine::node::live_strategy_node::if_else_node::condition::Case;
use crate::strategy_engine::node::node_types::NodeType;
use types::strategy::{LiveStrategyConfig, BacktestConfig, SimulatedConfig, TradeMode};
use crate::strategy_engine::node::live_strategy_node::live_data_node::live_data_node_context::*;
use crate::strategy_engine::node::live_strategy_node::indicator_node::indicator_node_type::*;
use crate::strategy_engine::node::live_strategy_node::if_else_node::if_else_node_type::*;
use crate::strategy_engine::node::live_strategy_node::order_node::order_node_types::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;

impl LiveStrategyFunction {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        node_config: Value,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Result<(), String> {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                Self::add_start_node(graph, node_indices, node_config, event_publisher).await;
                Ok(())
            }
            // 指标节点
            NodeType::IndicatorNode => {
                Self::add_indicator_node(graph, node_indices, node_config, event_publisher, response_event_receiver).await;
                Ok(())
                
            }
            // 实时数据节点
            NodeType::LiveDataNode => {
                Self::add_live_data_node(graph, node_indices, node_config, event_publisher, market_event_receiver, response_event_receiver).await;
                Ok(())
                
            }
            // 条件分支节点
            NodeType::IfElseNode => {
                Self::add_if_else_node(graph,node_indices,node_config,event_publisher).await;
                Ok(())
            }
            // 订单节点
            NodeType::OrderNode => {
                Self::add_order_node(graph, node_indices, node_config, event_publisher, response_event_receiver, exchange_engine, database, heartbeat).await;
                Ok(())
            }
            // 持仓节点
            NodeType::PositionNode => {
                Self::add_position_node(graph, node_indices, node_config, event_publisher, response_event_receiver, exchange_engine, database, heartbeat).await;
                Ok(())
                
            }
            // 获取变量节点
            NodeType::GetVariableNode => {
                Self::add_get_variable_node(graph, node_indices, node_config, event_publisher, response_event_receiver, exchange_engine, heartbeat, database).await;
                Ok(())
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
                Err("不支持的节点类型".to_string())
            }
            
        }

    }



    
    
}
