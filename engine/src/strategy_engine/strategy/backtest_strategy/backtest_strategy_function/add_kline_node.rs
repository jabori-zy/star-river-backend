use super::BacktestStrategyFunction;
use tokio::sync::Mutex;
use crate::strategy_engine::node::backtest_strategy_node::kline_node::KlineNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::node::backtest_strategy_node::kline_node::kline_node_type::KlineNodeBacktestConfig;
use std::sync::Arc;
use types::cache::key::KlineKey;
use event_center::EventReceiver;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::BacktestDataSource;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::mpsc;
use event_center::command::backtest_strategy_command::StrategyCommand;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;

impl BacktestStrategyFunction {
    pub async fn add_kline_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        ) -> Result<(), String> {

            let node_id = node_config["id"].as_str().unwrap(); // 节点id


            let strategy_command_rx = {
                let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);
                let strategy_context_guard = context.read().await;
                let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
                strategy_command_publisher.add_sender(node_id.to_string(), strategy_command_tx).await;
                strategy_command_rx
            };
            
            let (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, strategy_keys, play_index_watch_rx) = {
                let strategy_context_guard = context.read().await;
                let event_publisher = strategy_context_guard.event_publisher.clone();
                let command_publisher = strategy_context_guard.command_publisher.clone();
                let command_receiver = strategy_context_guard.command_receiver.clone();
                let heartbeat = strategy_context_guard.heartbeat.clone();
                let virtual_trading_system = strategy_context_guard.virtual_trading_system.clone();
                let strategy_keys = strategy_context_guard.keys.clone();
                let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
                (event_publisher, command_publisher, command_receiver, heartbeat, virtual_trading_system, strategy_keys, play_index_watch_rx)
            };






            let node_data = node_config["data"].clone();
            let strategy_id = node_data["strategyId"].as_i64().unwrap(); // 策略id
            
            let node_name = node_data["nodeName"].as_str().unwrap_or_default(); // 节点名称
            // 解析策略配置
            let kline_config_json = node_data["backtestConfig"].clone();
            if kline_config_json.is_null() {
                return Err("backtestConfig is null".to_string());
            }

            let backtest_config = serde_json::from_value::<KlineNodeBacktestConfig>(kline_config_json).unwrap();

            match backtest_config.data_source {
                BacktestDataSource::Exchange => {

                    let exchange = backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
                    let time_range = backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

                    for symbol_config in backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbols.iter() {
                        let backtest_kline_cache_key = KlineKey::new(
                            exchange.clone(), 
                            symbol_config.symbol.clone(), 
                            symbol_config.interval.clone(), 
                            Some(time_range.start_date.to_string()),
                            Some(time_range.end_date.to_string())
                        );
                        // 添加到策略缓存key列表中
                        let mut strategy_keys_guard = strategy_keys.write().await;
                        strategy_keys_guard.push(backtest_kline_cache_key.clone().into());
                        // 添加到虚拟交易系统中
                        let mut virtual_trading_system_guard = virtual_trading_system.lock().await;
                        virtual_trading_system_guard.add_kline_key(backtest_kline_cache_key);
                    }
                    
                }
                _ => {
                    return Err("data_source is not supported".to_string());
                }
            }

            

            let mut node = KlineNode::new(
                strategy_id as i32,
                node_id.to_string(), 
                node_name.to_string(), 
                backtest_config,
                event_publisher,
                command_publisher,
                command_receiver,
                market_event_receiver,
                response_event_receiver,
                heartbeat,
                node_command_sender,
                Arc::new(Mutex::new(strategy_command_rx)),
                strategy_inner_event_receiver,
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