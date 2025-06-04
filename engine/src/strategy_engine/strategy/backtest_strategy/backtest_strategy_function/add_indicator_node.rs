use super::BacktestStrategyFunction;
use crate::strategy_engine::node::BacktestNodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use tokio::sync::broadcast;
use types::market::{Exchange, KlineInterval};
use types::indicator::IndicatorConfig;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::IndicatorNode;
use event_center::{Event, EventPublisher};
use types::strategy::TradeMode;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::indicator_node_type::{IndicatorNodeBacktestConfig, ExchangeConfig, FileConfig};
use std::str::FromStr;
use types::cache::CacheKey;
use types::cache::cache_key::IndicatorCacheKey;
use types::cache::cache_key::BacktestKlineCacheKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::{BacktestDataSource, TimeRange};
use types::cache::cache_key::BacktestIndicatorCacheKey;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;

impl BacktestStrategyFunction {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<CacheKey>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        strategy_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {
        let node_data = node_config["data"].clone();
        let strategy_id = node_data["strategyId"].as_i64().unwrap();
        let node_id = node_config["id"].as_str().unwrap();
        let node_name = node_data["nodeName"].as_str().unwrap_or_default();

        
        let backtest_config_json = node_data["backtestConfig"].clone();
        if backtest_config_json.is_null() {
            return Err("backtestConfig is null".to_string());
        };
        
        // 解析指标类型
        let indicator_type = node_data["indicatorType"].as_str().unwrap_or_default(); // 指标类型
        // 解析指标配置
        let indicator_config_json = backtest_config_json["indicatorConfig"].clone();
        let indicator_config = IndicatorConfig::new(indicator_type, &indicator_config_json);
        let exchange_config_json = backtest_config_json["exchangeConfig"].clone();
        // 如果交易所配置不为空，则使用交易所配置
        let exchange_config = if !exchange_config_json.is_null() {
            let exchange_config = serde_json::from_value::<ExchangeConfig>(exchange_config_json).unwrap();
            let kline_cache_key = CacheKey::BacktestKline(BacktestKlineCacheKey::new(
                exchange_config.exchange.clone(),
                exchange_config.symbol.clone(),
                exchange_config.interval.clone(),
                exchange_config.time_range.start_date.clone().to_string(),
                exchange_config.time_range.end_date.clone().to_string(),
            ));
            let cache_key = BacktestIndicatorCacheKey::new(
                kline_cache_key,
                indicator_config.clone(),
            );
            cache_keys.push(cache_key.into());
            Some(exchange_config)
        } else {
            None
        };

        let backtest_config = IndicatorNodeBacktestConfig {
            data_source: BacktestDataSource::from_str(backtest_config_json["dataSource"].as_str().unwrap_or_default()).unwrap(),
            exchange_config: exchange_config,
            file_config: None,
            indicator_config: indicator_config,
        };

        

        let mut node = IndicatorNode::new(
            strategy_id as i32,
            node_id.to_string(), 
            node_name.to_string(), 
            backtest_config,
            event_publisher,
            command_publisher,
            command_receiver,
            response_event_receiver,
            strategy_command_sender,
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
