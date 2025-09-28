use super::super::super::node_message::futures_order_node_log_message::*;
use super::super::futures_order_node_types::*;
use super::FuturesOrderNodeContext;
use crate::backtest_strategy_engine::node::node_context::BacktestNodeContextTrait;
use event_center::event::strategy_event::{StrategyRunningLogEvent, StrategyRunningLogSource, StrategyRunningLogType};
use snafu::OptionExt;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::futures_order_node_error::*;

impl FuturesOrderNodeContext {
    pub(super) async fn create_order(&mut self, order_config: &FuturesOrderConfig) -> Result<(), FuturesOrderNodeError> {
        // 如果当前是正在处理订单的状态，或者未成交的订单列表不为空，则不创建订单
        if !self.can_create_order(&order_config.input_handle_id).await {
            tracing::warn!("{}: 当前正在处理订单, 跳过", self.get_node_name());
            let message = ProcessingOrderMsg::new(order_config.order_config_id);
            let current_time = self.get_current_time().await.unwrap();
            let log_event = StrategyRunningLogEvent::warn(
                self.get_strategy_id().clone(),
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                StrategyRunningLogSource::Node,
                StrategyRunningLogType::ProcessingOrder,
                message.to_string(),
                serde_json::Value::Null,
                current_time,
            );
            let _ = self.get_strategy_output_handle().send(log_event.into());
            return Err(CannotCreateOrderSnafu.build());
        }

        // 将input_handle_id的is_processing_order设置为true
        self.set_is_processing_order(&order_config.input_handle_id, true).await;

        let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let exchange = self
            .node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        // 创建订单
        // 获取symbol的point
        let point = self.symbol_info
            .iter()
            .find(|s| s.name == order_config.symbol)
            .context(SymbolInfoNotFoundSnafu { symbol: order_config.symbol.clone() })?
            .point();
        let create_order_result = virtual_trading_system_guard.create_order(
            self.get_strategy_id().clone(),
            self.get_node_id().clone(),
            order_config.order_config_id,
            order_config.symbol.clone(),
            exchange,
            order_config.price,
            order_config.order_side.clone(),
            order_config.order_type.clone(),
            order_config.quantity,
            order_config.tp,
            order_config.sl,
            order_config.tp_type.clone(),
            order_config.sl_type.clone(),
            Some(point as f64),
        );

        drop(virtual_trading_system_guard);

        if let Err(e) = create_order_result {
            // 如果创建订单失败，则直接重置is_processing_order
            self.set_is_processing_order(&order_config.input_handle_id, false).await;

            return Err(e.into());
        }

        Ok(())
    }
}
