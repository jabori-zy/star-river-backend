use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::{
    FuturesOrderNodeEvent, IfElseNodeEvent,
    futures_order_node_event::{TransactionCreatedEvent, TransactionCreatedPayload},
};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    event::node_common_event::{CommonEvent, NodeRunningLogEvent},
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use virtual_trading::{
    event::VtsEvent,
    types::{VirtualOrder, VirtualTransaction},
};

use super::FuturesOrderNodeContext;
use crate::{
    node::{
        node_command::{
            BacktestNodeCommand, GetFuturesOrderConfigRespPayload, GetFuturesOrderConfigResponse, NodeResetRespPayload, NodeResetResponse,
        },
        node_error::FuturesOrderNodeError,
        node_event::BacktestNodeEvent,
        node_message::futures_order_node_log_message::{OrderCanceledMsg, OrderCreatedMsg, OrderFilledMsg},
    },
    node_catalog::futures_order_node::context::config_filter::{filter_case_trigger_configs, filter_else_trigger_configs},
};

#[async_trait]
impl NodeEventHandlerExt for FuturesOrderNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn handle_source_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), Self::Error> {
        match node_event {
            BacktestNodeEvent::Common(common_evt) => {
                if let CommonEvent::Trigger(_trigger_evt) = common_evt {
                    tracing::debug!("@[{}] received trigger event", self.node_name());
                    // self.handle_trigger_event_for_independent_order(order_config_id).await?;
                }
                Ok(())
            }
            BacktestNodeEvent::IfElseNode(ifelse_node_event) => {
                self.handle_ifelse_node_event(ifelse_node_event).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let mut is_processing_order = self.is_processing_order.write().await;
                    is_processing_order.clear();
                    // Reset unfilled_virtual_order
                    let mut unfilled_virtual_order = self.unfilled_virtual_order.write().await;
                    unfilled_virtual_order.clear();
                    // Reset virtual_order_history
                    let mut virtual_order_history = self.virtual_order_history.write().await;
                    virtual_order_history.clear();

                    let payload = NodeResetRespPayload;
                    let response = NodeResetResponse::success(self.node_id().clone(), self.node_name().clone(), payload);
                    cmd.respond(response);
                }
            }
            BacktestNodeCommand::GetFuturesOrderConfig(cmd) => {
                tracing::debug!("@[{}] received get futures order config command", self.node_name());
                if cmd.node_id() == self.node_id() {
                    let futures_order_node_config = self.node_config.clone();
                    let payload = GetFuturesOrderConfigRespPayload::new(futures_order_node_config);
                    let response = GetFuturesOrderConfigResponse::success(self.node_id().clone(), self.node_name().clone(), payload);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}

impl FuturesOrderNodeContext {
    /// Send trigger event using the first connected output_handle

    pub async fn handle_node_event_for_independent_order(
        &mut self,
        node_event: BacktestNodeEvent,
        order_config_id: i32,
    ) -> Result<(), FuturesOrderNodeError> {
        // tracing::debug!("{}: Received event: {:?}", self.node_name(), node_event);
        match node_event {
            BacktestNodeEvent::Common(common_evt) => {
                if let CommonEvent::Trigger(_trigger_evt) = common_evt {
                    self.handle_trigger_event_for_independent_order(order_config_id).await?;
                }
                Ok(())
            }
            BacktestNodeEvent::IfElseNode(ifelse_node_event) => {
                self.handle_ifelse_node_event(ifelse_node_event).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn handle_trigger_event_for_independent_order(&mut self, order_config_id: i32) -> Result<(), FuturesOrderNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.cycle_id());
        let phase_name = format!("handle trigger event for specific order");
        cycle_tracker.start_phase(&phase_name);

        if self.is_leaf_node() {
            self.send_execute_over_event(
                Some(order_config_id),
                Some("handle trigger event for specific order".to_string()),
                Some(self.strategy_time()),
            )?;
        } else {
            self.independent_order_send_trigger_event(order_config_id, Some("handle trigger event for specific order".to_string()))
                .await?;
        }

        cycle_tracker.end_phase(&phase_name);
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    async fn handle_ifelse_node_event(&mut self, ifelse_node_event: IfElseNodeEvent) -> Result<(), FuturesOrderNodeError> {
        match ifelse_node_event {
            IfElseNodeEvent::CaseTrue(case_true) => {
                let mut cycle_tracker = CycleTracker::new(self.cycle_id());

                let phase_name = format!("handle case true for order");
                cycle_tracker.start_phase(&phase_name);

                // Get order configuration based on input_handle_id
                let config_ids = filter_case_trigger_configs(
                    self.node_config.futures_order_configs.iter(),
                    case_true.case_id,
                    case_true.node_id(),
                );

                for config_id in config_ids {
                    // if create order failed, send trigger event, no block strategy loop
                    self.create_order(config_id).await?;
                    if self.is_leaf_node() {
                        self.send_execute_over_event(
                            Some(config_id),
                            Some("create order success".to_string()),
                            Some(self.strategy_time()),
                        )?;
                    } else {
                        self.independent_order_send_trigger_event(config_id, Some("create order success".to_string()))
                            .await?;
                    }
                }
                cycle_tracker.end_phase(&phase_name);
                let completed_tracker = cycle_tracker.end();
                self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                    .await?;
                Ok(())
            }
            IfElseNodeEvent::CaseFalse(case_false) => {
                let config_ids = filter_case_trigger_configs(
                    self.node_config.futures_order_configs.iter(),
                    case_false.case_id,
                    case_false.node_id(),
                );

                for config_id in config_ids {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(
                            Some(config_id),
                            Some("handle case false event for order".to_string()),
                            Some(self.strategy_time()),
                        )?;
                    } else {
                        self.independent_order_send_trigger_event(config_id, Some("handle case false event for order".to_string()))
                            .await?;
                    }
                }

                Ok(())
            }
            IfElseNodeEvent::ElseTrue(else_true) => {
                let mut cycle_tracker = CycleTracker::new(self.cycle_id());

                let phase_name = format!("handle case true for order");
                cycle_tracker.start_phase(&phase_name);

                // Get order configuration based on input_handle_id
                let config_ids = filter_else_trigger_configs(self.node_config.futures_order_configs.iter(), else_true.node_id());

                for config_id in config_ids {
                    // if create order failed, send trigger event, no block strategy loop
                    self.create_order(config_id).await?;
                    if self.is_leaf_node() {
                        self.send_execute_over_event(
                            Some(config_id),
                            Some("create order success".to_string()),
                            Some(self.strategy_time()),
                        )?;
                    } else {
                        self.independent_order_send_trigger_event(config_id, Some("create order success".to_string()))
                            .await?;
                    }
                }

                cycle_tracker.end_phase(&phase_name);
                let completed_tracker = cycle_tracker.end();
                self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                    .await?;
                Ok(())
            }
            IfElseNodeEvent::ElseFalse(else_false) => {
                let config_ids = filter_else_trigger_configs(self.node_config.futures_order_configs.iter(), else_false.node_id());

                for config_id in config_ids {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(
                            Some(config_id),
                            Some("handle case false event for order".to_string()),
                            Some(self.strategy_time()),
                        )?;
                    } else {
                        self.independent_order_send_trigger_event(config_id, Some("handle case false event for order".to_string()))
                            .await?;
                    }
                }

                Ok(())
            }
        }
    }

    pub(super) async fn send_order_status_event(
        &self,
        virtual_order: VirtualOrder,
        vts_event: &VtsEvent,
    ) -> Result<(), FuturesOrderNodeError> {
        let order_status = &virtual_order.order_status;
        let node_id = self.node_id().clone();

        let strategy_output_handle = self.strategy_bound_handle();
        // Send to strategy_output_handle first to ensure event is always sent
        let order_event =
            self.generate_order_node_event(strategy_output_handle.output_handle_id().clone(), virtual_order.clone(), vts_event);
        if let Some(order_event) = order_event {
            self.strategy_bound_handle_send(order_event.into())?;
        }

        if self.is_leaf_node() {
            // self.send_execute_over_event(Some(virtual_order.order_config_id), Some("send order status event".to_string()), Some(self.strategy_time()))?;
            Ok(())
        } else {
            let order_status_output_handle_id =
                format!("{}_{}_output_{}", node_id, order_status.to_string(), virtual_order.order_config_id);
            let order_status_output_handle = self.output_handle(&order_status_output_handle_id)?;

            let order_event = self.generate_order_node_event(
                order_status_output_handle.output_handle_id().clone(),
                virtual_order.clone(),
                vts_event,
            );
            if let Some(order_event) = order_event {
                order_status_output_handle.send(order_event.into())?;
            }
            // The all_output_handle_id corresponding to the order config id
            let order_all_status_output_handle_id = format!("{}_all_status_output_{}", node_id, virtual_order.order_config_id);
            let order_all_status_output_handle = self.output_handle(&order_all_status_output_handle_id)?;
            let order_event = self.generate_order_node_event(
                order_all_status_output_handle.output_handle_id().clone(),
                virtual_order.clone(),
                vts_event,
            );
            if let Some(order_event) = order_event {
                order_all_status_output_handle.send(order_event.into())?;
            }
            Ok(())
        }
    }

    // Handle virtual trading system events
    pub(crate) async fn handle_vts_event(&mut self, virtual_trading_system_event: VtsEvent) -> Result<(), FuturesOrderNodeError> {
        let order: Option<&VirtualOrder> = match &virtual_trading_system_event {
            VtsEvent::FuturesOrderCreated(order)
            | VtsEvent::FuturesOrderFilled(order)
            | VtsEvent::FuturesOrderCanceled(order)
            | VtsEvent::TakeProfitOrderCreated(order)
            | VtsEvent::TakeProfitOrderFilled(order)
            | VtsEvent::TakeProfitOrderCanceled(order)
            | VtsEvent::StopLossOrderCreated(order)
            | VtsEvent::StopLossOrderFilled(order)
            | VtsEvent::StopLossOrderCanceled(order) => Some(order),
            _ => None,
        };

        if let Some(order) = order {
            if order.node_id == self.node_id().clone() {
                match virtual_trading_system_event {
                    VtsEvent::FuturesOrderCreated(_) => {
                        tracing::debug!("[{}] receive futures order created event", self.node_name());
                        self.add_unfilled_virtual_order(order.clone()).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderCreatedMsg::new(
                            order.order_id,
                            order.order_config_id,
                            order.open_price,
                            order.order_side.to_string(),
                        );
                        let log_event: CommonEvent = NodeRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            message.to_string(),
                            order.to_value()?,
                            order.create_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    VtsEvent::FuturesOrderFilled(_) => {
                        self.remove_unfilled_virtual_order(order.order_id).await;
                        self.add_virtual_order_history(order.clone()).await;
                        self.set_is_processing_order(order.order_config_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderFilledMsg::new(self.node_name().clone(), order.order_id, order.quantity, order.open_price);
                        let log_event: CommonEvent = NodeRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    VtsEvent::FuturesOrderCanceled(_) => {
                        self.remove_unfilled_virtual_order(order.order_id).await;
                        self.add_virtual_order_history(order.clone()).await;
                        self.set_is_processing_order(order.order_config_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderCanceledMsg::new(self.node_name().clone(), order.order_id);
                        let log_event: CommonEvent = NodeRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    // Only send events
                    VtsEvent::TakeProfitOrderCreated(_)
                    | VtsEvent::TakeProfitOrderFilled(_)
                    | VtsEvent::TakeProfitOrderCanceled(_)
                    | VtsEvent::StopLossOrderCreated(_)
                    | VtsEvent::StopLossOrderFilled(_)
                    | VtsEvent::StopLossOrderCanceled(_) => {
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                    }

                    _ => {}
                }
            }
        }

        let transaction: Option<&VirtualTransaction> = match &virtual_trading_system_event {
            VtsEvent::TransactionCreated(transaction) => Some(transaction),
            _ => None,
        };

        if let Some(transaction) = transaction {
            if transaction.node_id == self.node_id().clone() {
                let input_handle_id = format!("{}_input_{}", self.node_id(), transaction.order_config_id);
                self.add_virtual_transaction_history(transaction.clone()).await;
                let payload = TransactionCreatedPayload::new(transaction.clone());
                let transaction_event: FuturesOrderNodeEvent = TransactionCreatedEvent::new_with_time(
                    self.cycle_id(),
                    self.node_id().clone(),
                    self.node_name().clone(),
                    input_handle_id.clone(),
                    self.strategy_time(),
                    payload,
                )
                .into();
                self.strategy_bound_handle_send(transaction_event.into())?;
            }
        }

        Ok(())
    }
}
