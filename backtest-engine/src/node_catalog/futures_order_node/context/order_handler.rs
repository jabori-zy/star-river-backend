use snafu::{OptionExt, ResultExt};
use strategy_core::{
    event::node_common_event::{CommonEvent, NodeRunningLogEvent},
    node::context_trait::{NodeCommunicationExt, NodeInfoExt, NodeRelationExt},
};
use tokio::sync::oneshot;
use virtual_trading::{
    command::{CreateOrderCmdPayload, CreateOrderCommand, VtsResponse},
    error::{CommandSendFailedSnafu, ResponseRecvFailedSnafu},
};

use super::{super::futures_order_node_types::*, FuturesOrderNodeContext};
use crate::node::{
    node_error::{
        FuturesOrderNodeError,
        futures_order_node_error::{CannotCreateOrderSnafu, SymbolInfoNotFoundSnafu},
    },
    node_message::futures_order_node_log_message::ProcessingOrderMsg,
};

impl FuturesOrderNodeContext {
    // create a virtual order
    pub(super) async fn create_order(&mut self, config_id: i32) -> Result<(), FuturesOrderNodeError> {
        tracing::debug!("@[{}] creating order for config: {:?}", self.node_name(), config_id);
        // 如果当前是正在处理订单的状态，或者未成交的订单列表不为空，则不创建订单
        if !self.can_create_order(&config_id).await {
            tracing::warn!("@[{}] config {:?} is processing order, skip", self.node_name(), config_id);
            if self.warn_log_send_count(&config_id).await < 3 {
                let message = ProcessingOrderMsg::new(config_id);
                let current_time = self.strategy_time();
                let log_event: CommonEvent = NodeRunningLogEvent::warn_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    self.node_id().clone(),
                    self.node_name().clone(),
                    message.to_string(),
                    None,
                    None,
                    current_time,
                )
                .into();
                self.strategy_bound_handle_send(log_event.into())?;
                self.increment_warn_log_send_count(config_id).await;
            }

            if self.is_leaf_node() {
                self.send_execute_over_event(Some(config_id), Some("is processing order".to_string()), Some(self.strategy_time()))?;
            } else {
                self.independent_order_send_trigger_event(config_id, Some("create order failed".to_string()))
                    .await?;
            }
            return Ok(());
        }
        // 将input_handle_id的is_processing_order设置为true
        self.set_is_processing_order(config_id, true).await;

        // let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let order_config = self.node_config.find_order_config(config_id)?;
        // 创建订单
        // 获取symbol的point
        let point = self
            .symbol_info
            .iter()
            .find(|s| s.name == order_config.symbol)
            .context(SymbolInfoNotFoundSnafu {
                symbol: order_config.symbol.clone(),
            })?
            .point();

        let payload = CreateOrderCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            self.node_name().clone(),
            config_id,
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

        let (tx, rx) = oneshot::channel();
        let cmd = CreateOrderCommand::new(tx, payload);
        self.vts_command_sender.send(cmd.into()).await.context(CommandSendFailedSnafu {})?;

        let response = rx.await.context(ResponseRecvFailedSnafu {})?;
        match response {
            VtsResponse::Success { .. } => {
                return Ok(());
            }
            VtsResponse::Fail { error, .. } => {
                self.set_is_processing_order(config_id, false).await;
                return Err(error.into());
            }
        }
    }
}
