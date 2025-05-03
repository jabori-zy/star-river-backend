use crate::strategy_engine::strategy::Strategy;
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
use crate::strategy_engine::node::if_else_node::condition::Case;
use crate::strategy_engine::node::node_types::NodeType;
use types::strategy::{LiveConfig, BacktestConfig, SimulatedConfig, TradeMode};
use crate::strategy_engine::node::live_data_node::live_data_node_context::*;
use crate::strategy_engine::node::indicator_node::indicator_node_type::*;
use crate::strategy_engine::node::if_else_node::if_else_node_type::*;
use crate::strategy_engine::node::order_node::order_node_types::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use crate::strategy_engine::node::position_node::position_node_types::*;

impl Strategy {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        trade_mode: TradeMode,
        node_config: &Value,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                let node_data = node_config["data"].clone();
                let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
                let node_id = node_config["id"].as_str().unwrap(); // 节点id
                let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
                // tracing::debug!("开始节点数据: {:?}, {:?}, {:?}", node_data["liveConfig"], node_data["backtestConfig"], node_data["simulatedConfig"]);
                // 解析策略配置
                let live_config = match node_data["liveConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<LiveConfig>(node_data["liveConfig"].clone()).unwrap()),
                };
                let backtest_config = match node_data["backtestConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<BacktestConfig>(node_data["backtestConfig"].clone()).unwrap()),
                };
                let simulated_config = match node_data["simulatedConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<SimulatedConfig>(node_data["simulatedConfig"].clone()).unwrap()),
                };
                let event_publisher = event_publisher.clone(); // 事件发布者
                Self::add_start_node(
                    graph, 
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(),
                    trade_mode,
                    live_config,
                    backtest_config,
                    simulated_config,
                    event_publisher,
                ).await;
            }
            // 指标节点
            NodeType::IndicatorNode => {
                let node_data = node_config["data"].clone();
                let strategy_id = node_data["strategyId"].as_i64().unwrap();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();

                let indicator_type = node_data["indicatorType"].as_str().unwrap_or_default(); // 指标类型
                let mut indicator = Indicators::from_str(indicator_type).unwrap(); // 转换成指标
                let live_config = match node_data["liveConfig"].is_null() {
                    true => None,
                    false => {
                        let live_config_json = node_data["liveConfig"].clone();
                        let indicator_config = live_config_json["indicatorConfig"].clone();
                        indicator.update_config(&indicator_config); // 更新指标配置
                        let symbol = live_config_json["symbol"].as_str().unwrap_or_default().to_string();
                        let interval = KlineInterval::from_str(live_config_json["interval"].as_str().unwrap_or_default()).unwrap();
                        let exchange = Exchange::from_str(live_config_json["exchange"].as_str().unwrap_or_default()).unwrap();
                        let indicator_node_live_config = IndicatorNodeLiveConfig {
                            indicator: indicator.clone(),
                            symbol,
                            interval,
                            exchange,
                        };
                        Some(indicator_node_live_config)
                    },
                };
                let backtest_config = match node_data["backtestConfig"].is_null() {
                    true => None,
                    false => {
                        let backtest_config_json = node_data["backtestConfig"].clone();
                        let indicator_config = backtest_config_json["indicatorConfig"].clone();
                        indicator.update_config(&indicator_config); // 更新指标配置
                        let symbol = backtest_config_json["symbol"].as_str().unwrap_or_default().to_string();
                        let interval = KlineInterval::from_str(backtest_config_json["interval"].as_str().unwrap_or_default()).unwrap();
                        let exchange = Exchange::from_str(backtest_config_json["exchange"].as_str().unwrap_or_default()).unwrap();
                        let indicator_node_backtest_config = IndicatorNodeBacktestConfig {
                            indicator: indicator.clone(),
                            symbol,
                            interval,
                            exchange,
                        };
                        Some(indicator_node_backtest_config)
                    },
                };
                let simulated_config = match node_data["simulatedConfig"].is_null() {
                    true => None,
                    false => {
                        let simulated_config_json = node_data["simulatedConfig"].clone();
                        let indicator_config = simulated_config_json["indicatorConfig"].clone();
                        indicator.update_config(&indicator_config); // 更新指标配置
                        let symbol = simulated_config_json["symbol"].as_str().unwrap_or_default().to_string();
                        let interval = KlineInterval::from_str(simulated_config_json["interval"].as_str().unwrap_or_default()).unwrap();
                        let exchange = Exchange::from_str(simulated_config_json["exchange"].as_str().unwrap_or_default()).unwrap();
                        let indicator_node_simulate_config = IndicatorNodeSimulateConfig {
                            indicator: indicator.clone(),
                            symbol,
                            interval,
                            exchange,
                        };
                        Some(indicator_node_simulate_config)
                    },
                };
                
                let response_event_receiver = response_event_receiver.resubscribe();
                Self::add_indicator_node(
                    graph, 
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(),
                    live_config,
                    backtest_config,
                    simulated_config,
                    trade_mode,
                    event_publisher.clone(),
                    response_event_receiver,
                ).await;
                
            }
            // 实时数据节点
            NodeType::LiveDataNode => {
                let node_data = node_config["data"].clone();
                let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
                let node_id = node_config["id"].as_str().unwrap(); // 节点id
                let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
                // let account_id = node_data["accountId"].as_i64().unwrap(); // 账户id
                // let exchange = node_data["exchange"].as_str().unwrap(); // 交易所
                // let symbol = node_data["symbol"].as_str().unwrap(); // 交易对
                // let interval = node_data["interval"].as_str().unwrap(); // 时间周期
                let event_publisher = event_publisher.clone(); // 事件发布者
                let market_event_receiver = market_event_receiver.resubscribe(); // 市场事件接收者
                let response_event_receiver = response_event_receiver.resubscribe(); // 响应事件接收者
                // k线频率设置
                let frequency = 2000;
                tracing::debug!("实时数据节点数据: {:?}", node_data);
                // 解析策略配置
                let live_config = match node_data["liveConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<LiveDataNodeLiveConfig>(node_data["liveConfig"].clone()).unwrap()),
                };
                let backtest_config = match node_data["backtestConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<LiveDataNodeBacktestConfig>(node_data["backtestConfig"].clone()).unwrap()),
                };
                let simulated_config = match node_data["simulatedConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<LiveDataNodeSimulateConfig>(node_data["simulatedConfig"].clone()).unwrap()),
                };

                Self::add_live_data_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    trade_mode,
                    live_config,
                    backtest_config,
                    simulated_config,
                    event_publisher,
                    market_event_receiver,
                    response_event_receiver,
                ).await;
                
            }
            // 条件分支节点
            NodeType::IfElseNode => {
                let node_data = node_config["data"].clone();
                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let strategy_id = node_data["strategyId"].as_i64().unwrap();
                // let cases: Vec<Case> = serde_json::from_value(node_data["cases"].clone())
                //     .unwrap_or_else(|e| panic!("Failed to parse cases: {}", e));
                let live_config = match node_data["liveConfig"].is_null() {
                    true => None,
                    false => {
                        let cases: Vec<Case> = serde_json::from_value(node_data["liveConfig"]["cases"].clone()).unwrap();
                        let if_else_node_live_config = IfElseNodeLiveConfig {
                            cases: cases.clone(),
                        };
                        tracing::debug!("条件分支节点数据: {:?}", if_else_node_live_config);
                        Some(if_else_node_live_config)
                    },
                };
                let backtest_config = match node_data["backtestConfig"].is_null() {
                    true => None,
                    false => {
                        let cases: Vec<Case> = serde_json::from_value(node_data["backtestConfig"]["cases"].clone()).unwrap();
                        let if_else_node_backtest_config = IfElseNodeBacktestConfig {
                            cases: cases.clone(),
                        };
                        Some(if_else_node_backtest_config)
                    },
                };
                let simulate_config = match node_data["simulatedConfig"].is_null() {
                    true => None,
                    false => {
                        let cases: Vec<Case> = serde_json::from_value(node_data["simulatedConfig"]["cases"].clone()).unwrap();
                        let if_else_node_simulate_config = IfElseNodeSimulateConfig {
                            cases: cases.clone(),
                        };
                        Some(if_else_node_simulate_config)
                    },
                };

                Self::add_if_else_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id.to_string(),
                    node_name.to_string(),
                    trade_mode,
                    live_config,
                    backtest_config,
                    simulate_config,
                    event_publisher,
                ).await;
            }
            // 订单节点
            NodeType::OrderNode => {
                let node_data = node_config["data"].clone(); // 节点数据

                let node_id = node_config["id"].as_str().unwrap().to_string(); // 节点id
                let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
                let node_name = node_data["nodeName"].as_str().unwrap().to_string(); // 节点名称
                let live_config = match node_data["liveConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<OrderNodeLiveConfig>(node_data["liveConfig"].clone()).unwrap()),
                };
                let backtest_config = match node_data["backtestConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<OrderNodeBacktestConfig>(node_data["backtestConfig"].clone()).unwrap()),
                };
                let simulate_config = match node_data["simulatedConfig"].is_null() {
                    true => None,
                    false => Some(serde_json::from_value::<OrderNodeSimulateConfig>(node_data["simulatedConfig"].clone()).unwrap()),
                };

                let event_publisher = event_publisher.clone();
                let response_event_receiver = response_event_receiver.resubscribe();
                Self::add_order_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id,
                    node_name,
                    trade_mode,
                    live_config,
                    simulate_config,
                    backtest_config,
                    event_publisher,
                    response_event_receiver,
                    exchange_engine,
                    database,
                    heartbeat,
                ).await;
                
            }
            // 持仓节点
            NodeType::PositionNode => {
                Self::add_position_node(
                    graph,
                    node_indices,
                    trade_mode,
                    node_config.clone(),
                    event_publisher,
                    response_event_receiver,
                    exchange_engine,
                    database,
                    heartbeat,
                ).await;
                
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
            }
            
        }

    }



    
    
}
