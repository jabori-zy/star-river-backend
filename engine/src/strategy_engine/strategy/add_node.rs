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
use types::order::{OrderType, OrderSide};
use crate::strategy_engine::node::order_node::order_node_types::OrderConfig;
use types::strategy::{LiveConfig, BacktestConfig, SimulatedConfig, TradeMode};
use crate::strategy_engine::node::live_data_node::live_data_node_context::{LiveDataNodeLiveConfig, LiveDataNodeBacktestConfig, LiveDataNodeSimulateConfig};


impl Strategy {
    pub async fn add_node(
        graph: &mut Graph<Box<dyn NodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        trade_mode: TradeMode,
        node_config: &Value,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
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

                let indicator_name = node_data["indicatorName"].as_str().unwrap_or_default(); // 指标名称
                let mut indicator = Indicators::from_str(indicator_name).unwrap(); // 转换成指标
                let indicator_config = node_data["indicatorConfig"].clone(); // 指标配置
                indicator.update_config(&indicator_config); // 更新指标配置

                let exchange = Exchange::Binance; // 交易所
                let symbol = "BTCUSDT".to_string();
                let interval = KlineInterval::Minutes1;
                
                let response_event_receiver = response_event_receiver.resubscribe();
                Self::add_indicator_node(
                    graph, 
                    node_indices,
                    strategy_id,
                    node_id.to_string(), 
                    node_name.to_string(), 
                    exchange, 
                    symbol, 
                    interval, 
                    indicator,
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
                    1,
                    Exchange::Binance, 
                    "BTCUSDT".to_string(), 
                    KlineInterval::Minutes1, 
                    frequency,
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
                let cases: Vec<Case> = serde_json::from_value(node_data["cases"].clone())
                    .unwrap_or_else(|e| panic!("Failed to parse cases: {}", e));
                Self::add_if_else_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id.to_string(),
                    node_name.to_string(),
                    cases,
                    trade_mode,
                    event_publisher,
                ).await;
            }
            // 订单节点
            NodeType::OrderNode => {
                let node_data = node_config["data"].clone();

                let node_id = node_config["id"].as_str().unwrap();
                let node_name = node_data["nodeName"].as_str().unwrap_or_default();
                let account_id = node_data["accountId"].as_i64().unwrap();
                let exchange = node_data["exchange"].as_str().unwrap();
                let symbol = node_data["symbol"].as_str().unwrap();
                let strategy_id = node_data["strategyId"].as_i64().unwrap();
                let event_publisher = event_publisher.clone();
                let response_event_receiver = response_event_receiver.resubscribe();
                let order_info = node_data["orderRequest"].clone();
                let order_config = OrderConfig {
                    order_type: OrderType::from_str(order_info["orderType"].as_str().unwrap()).unwrap(),
                    order_side: OrderSide::from_str(order_info["orderSide"].as_str().unwrap()).unwrap(),
                    quantity: order_info["quantity"].as_f64().unwrap(),
                    price: order_info["price"].as_f64().unwrap(),
                    tp: order_info["tp"].as_f64(),
                    sl: order_info["sl"].as_f64(),
                    comment: "111".to_string(),
                };
                Self::add_order_node(
                    graph,
                    node_indices,
                    strategy_id,
                    node_id.to_string(),
                    node_name.to_string(),
                    account_id as i32,
                    Exchange::from_str(exchange).unwrap(),
                    symbol.to_string(),
                    order_config,
                    trade_mode,
                    event_publisher,
                    response_event_receiver,
                ).await;
                
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
            }
            
        }

    }



    
    
}
