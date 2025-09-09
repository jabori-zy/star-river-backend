use super::BacktestStrategyFunction;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::indicator_node_context::IndicatorNodeContext;
use crate::strategy_engine::node::backtest_strategy_node::indicator_node::IndicatorNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use event_center::command::backtest_strategy_command::StrategyCommand;
use event_center::EventReceiver;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::cache::key::IndicatorKey;
use types::cache::key::KlineKey;
use types::error::engine_error::strategy_engine_error::node_error::indicator_node_error::*;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;

impl BacktestStrategyFunction {
    pub async fn add_indicator_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: serde_json::Value,
        // response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), IndicatorNodeError> {
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<StrategyCommand>(100);

        let (strategy_keys, play_index_watch_rx) = {
            let strategy_context_guard = context.read().await;
            // let event_publisher = strategy_context_guard.event_publisher.clone();
            // let command_publisher = strategy_context_guard.command_publisher.clone();
            // let command_receiver = strategy_context_guard.command_receiver.clone();
            let strategy_keys = strategy_context_guard.keys.clone();
            let play_index_watch_rx = strategy_context_guard.play_index_watch_rx.clone();
            (strategy_keys, play_index_watch_rx)
        };

        let mut node = IndicatorNode::new(
            node_config,
            // event_publisher,
            // command_publisher,
            // command_receiver,
            // response_event_receiver,
            node_command_sender,
            Arc::new(Mutex::new(strategy_command_rx)),
            strategy_inner_event_receiver,
            play_index_watch_rx,
        )?;

        // update strategy keys
        let indicator_keys = {
            let node_context_rwlock = node.get_context();
            let node_context = node_context_rwlock.read().await;
            let node_context_guard = node_context
                .as_any()
                .downcast_ref::<IndicatorNodeContext>()
                .unwrap();
            let exchange_mode_config = node_context_guard
                .backtest_config
                .exchange_mode_config
                .as_ref()
                .unwrap();
            let selected_account = &exchange_mode_config.selected_account;
            let selected_symbol = &exchange_mode_config.selected_symbol;
            let time_range = &exchange_mode_config.time_range;
            let selected_indicators = &exchange_mode_config.selected_indicators;

            let kline_key = KlineKey::new(
                selected_account.exchange.clone(),
                selected_symbol.symbol.clone(),
                selected_symbol.interval.clone(),
                Some(time_range.start_date.to_string()),
                Some(time_range.end_date.to_string()),
            );
            let mut indicator_keys = vec![];
            selected_indicators.iter().for_each(|indicator| {
                let indicator_key =
                    IndicatorKey::new(kline_key.clone(), indicator.indicator_config.clone());
                indicator_keys.push(indicator_key);
            });
            indicator_keys
        }; // 读锁在这里释放

        let mut strategy_keys_guard = strategy_keys.write().await;
        strategy_keys_guard.extend(indicator_keys.iter().map(|key| key.clone().into()));
        drop(strategy_keys_guard); // 显式释放写锁

        // set default output handle
        let node_id = node.get_node_id().await;
        node.set_output_handle().await;

        let mut strategy_context_guard = context.write().await;

        let strategy_command_publisher = &strategy_context_guard.strategy_command_publisher;
        strategy_command_publisher
            .add_sender(node_id.to_string(), strategy_command_tx)
            .await;

        let node = Box::new(node);

        let node_index = strategy_context_guard.graph.add_node(node);
        strategy_context_guard
            .node_indices
            .insert(node_id.to_string(), node_index);

        Ok(())
    }
}
