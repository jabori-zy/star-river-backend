use super::BacktestStrategyFunction;
use crate::strategy_engine::node::BacktestNodeTrait;
use petgraph::{Graph, Directed};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use types::indicator::IndicatorConfig;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::IndicatorNode;
use event_center::EventPublisher;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::indicator_node_type::{IndicatorNodeBacktestConfig, ExchangeModeConfig,SelectedIndicator};
use std::str::FromStr;
use types::cache::Key;
use types::cache::key::BacktestKlineKey;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::{BacktestDataSource, TimeRange};
use types::cache::key::BacktestIndicatorKey;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use types::strategy::SelectedAccount;
use crate::strategy_engine::node::backtest_strategy_node::kline_node::kline_node_type::SelectedSymbol;
use super::super::StrategyCommandPublisher;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;

impl BacktestStrategyFunction {
    pub async fn add_indicator_node(
        graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>, 
        node_indices: &mut HashMap<String, NodeIndex>,
        cache_keys: &mut Vec<Key>,
        node_config: serde_json::Value,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_command_publisher: &mut StrategyCommandPublisher,
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

        tracing::debug!("backtest_config_json: {:?}", backtest_config_json);

        // 解析已配置的账户
        let selected_account_json = backtest_config_json["exchangeModeConfig"]["selectedAccount"].clone();
        let selected_account = serde_json::from_value::<SelectedAccount>(selected_account_json).unwrap();
        tracing::debug!("selected_account: {:?}", selected_account);

        // 解析已配置的symbol
        let selected_symbol_json = backtest_config_json["exchangeModeConfig"]["selectedSymbol"].clone();
        let selected_symbol = serde_json::from_value::<SelectedSymbol>(selected_symbol_json).unwrap();
        tracing::debug!("selected_symbol: {:?}", selected_symbol);
        
        // 已配置的指标列表
        let selected_indicators_array = backtest_config_json["exchangeModeConfig"]["selectedIndicators"].as_array().unwrap();
        let time_range_json = backtest_config_json["exchangeModeConfig"]["timeRange"].clone();
        let time_range = serde_json::from_value::<TimeRange>(time_range_json).unwrap();

        // 解析指标配置
        let mut selected_indicators = Vec::new();
        for ind_config in selected_indicators_array {
            tracing::debug!("ind_config: {:?}", ind_config);
            // 指标配置json
            let indicator_config_json = ind_config["indicatorConfig"].clone();
            let indicator_type = indicator_config_json["type"].as_str().unwrap_or_default();
            let indicator_config = IndicatorConfig::new(indicator_type, &indicator_config_json);
            let selected_indicator = SelectedIndicator {
                indicator_id: ind_config["indicatorId"].as_i64().unwrap() as i32,
                handle_id: ind_config["handleId"].as_str().unwrap_or_default().to_string(),
                indicator_config: indicator_config.clone(),
            };
            selected_indicators.push(selected_indicator);

            let kline_cache_key = Key::BacktestKline(BacktestKlineKey::new(
                selected_account.exchange.clone(),
                selected_symbol.symbol.clone(),
                selected_symbol.interval.clone(),
                time_range.start_date.to_string(),
                time_range.end_date.to_string(),
            ));

            let indicator_cache_key = Key::BacktestIndicator(BacktestIndicatorKey::new(
                kline_cache_key,
                indicator_config,
            ));
            cache_keys.push(indicator_cache_key.into());
        }
        tracing::debug!("selected_indicators: {:?}", selected_indicators);


        let exchange_mode_config = ExchangeModeConfig {
            selected_account: selected_account,
            selected_symbol: selected_symbol,
            selected_indicators: selected_indicators,
            time_range: time_range,
        };

        let backtest_config = IndicatorNodeBacktestConfig {
            data_source: BacktestDataSource::from_str(backtest_config_json["dataSource"].as_str().unwrap_or_default()).unwrap(),
            exchange_mode_config: Some(exchange_mode_config),
            file_mode_config: None,
        };
        tracing::debug!("backtest_config: {:?}", backtest_config);


        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
        strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;


        let mut node = IndicatorNode::new(
            strategy_id as i32,
            node_id.to_string(), 
            node_name.to_string(), 
            backtest_config,
            event_publisher,
            command_publisher,
            command_receiver,
            response_event_receiver,
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
