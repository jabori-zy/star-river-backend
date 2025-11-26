use async_trait::async_trait;
use event_center::Event;
use star_river_event::backtest_strategy::node_event::{
    FuturesOrderNodeEvent, IfElseNodeEvent,
    futures_order_node_event::{TransactionCreatedEvent, TransactionCreatedPayload},
};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    event::{
        node_common_event::CommonEvent,
        strategy_event::{StrategyRunningLogEvent, StrategyRunningLogSource, StrategyRunningLogType},
    },
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use virtual_trading::{
    event::VtsEvent,
    types::{VirtualOrder, VirtualTransaction},
};

use super::FuturesOrderNodeContext;
use crate::node::{
    node_command::{
        BacktestNodeCommand, GetFuturesOrderConfigRespPayload, GetFuturesOrderConfigResponse, NodeResetRespPayload, NodeResetResponse,
    },
    node_error::{FuturesOrderNodeError, futures_order_node_error::OrderConfigNotFoundSnafu},
    node_event::BacktestNodeEvent,
    node_message::futures_order_node_log_message::{OrderCanceledMsg, OrderCreatedMsg, OrderFilledMsg},
};

#[async_trait]
impl NodeEventHandlerExt for FuturesOrderNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn handle_source_node_event(&mut self, _node_event: BacktestNodeEvent) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn handle_command(&mut self, node_command: BacktestNodeCommand) -> Result<(), Self::Error> {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let mut is_processing_order = self.is_processing_order.write().await;
                    is_processing_order.clear();
                    // 重置unfilled_virtual_order
                    let mut unfilled_virtual_order = self.unfilled_virtual_order.write().await;
                    unfilled_virtual_order.clear();
                    // 重置virtual_order_history
                    let mut virtual_order_history = self.virtual_order_history.write().await;
                    virtual_order_history.clear();

                    let payload = NodeResetRespPayload;
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            BacktestNodeCommand::GetFuturesOrderConfig(cmd) => {
                tracing::debug!("@[{}] received get futures order config command", self.node_name());
                if cmd.node_id() == self.node_id() {
                    let futures_order_node_config = self.node_config.clone();
                    let payload = GetFuturesOrderConfigRespPayload::new(futures_order_node_config);
                    let response = GetFuturesOrderConfigResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

impl FuturesOrderNodeContext {
    /// 使用第一个有连接的output_handle发送trigger事件

    pub async fn handle_node_event_for_independent_order(
        &mut self,
        node_event: BacktestNodeEvent,
        order_config_id: i32,
    ) -> Result<(), FuturesOrderNodeError> {
        match node_event {
            BacktestNodeEvent::Common(common_evt) => {
                if let CommonEvent::Trigger(_trigger_evt) = common_evt {
                    self.handle_trigger_event_for_independent_order(order_config_id).await?;
                }
                Ok(())
            }
            BacktestNodeEvent::IfElseNode(ifelse_node_event) => {
                self.handle_ifelse_node_event_for_independent_order(ifelse_node_event, order_config_id)
                    .await?;
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
            self.send_execute_over_event(Some(order_config_id), Some(self.strategy_time()))?;
        } else {
            self.independent_order_send_trigger_event(order_config_id).await?;
        }

        cycle_tracker.end_phase(&phase_name);
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await?;
        Ok(())
    }

    async fn handle_ifelse_node_event_for_independent_order(
        &mut self,
        ifelse_node_event: IfElseNodeEvent,
        order_config_id: i32,
    ) -> Result<(), FuturesOrderNodeError> {
        match ifelse_node_event {
            IfElseNodeEvent::CaseTrue(_) | IfElseNodeEvent::ElseTrue(_) => {
                let mut cycle_tracker = CycleTracker::new(self.cycle_id());

                let phase_name = format!("handle condition match event for order {}", order_config_id);
                cycle_tracker.start_phase(&phase_name);

                // 根据input_handle_id获取订单配置
                let order_config = self
                    .node_config
                    .futures_order_configs
                    .iter()
                    .find(|config| config.order_config_id == order_config_id)
                    .ok_or(OrderConfigNotFoundSnafu { order_config_id }.build())?
                    .clone();

                // if create order failed, send trigger event, no block strategy loop
                if self.create_order(&order_config).await.is_err() {
                    self.independent_order_send_trigger_event(order_config_id).await?;
                }

                cycle_tracker.end_phase(&phase_name);
                let completed_tracker = cycle_tracker.end();
                self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                    .await?;
                Ok(())
            }
            IfElseNodeEvent::CaseFalse(_) | IfElseNodeEvent::ElseFalse(_) => {
                if self.is_leaf_node() {
                    self.send_execute_over_event(Some(order_config_id), Some(self.strategy_time()))?;
                } else {
                    self.independent_order_send_trigger_event(order_config_id).await?;
                }
                Ok(())
            }
        }
    }

    pub(super) async fn send_order_status_event(
        &self,
        virtual_order: VirtualOrder,
        event_type: &VtsEvent,
    ) -> Result<(), FuturesOrderNodeError> {
        let order_status = &virtual_order.order_status;
        let node_id = self.node_id().clone();

        let strategy_output_handle = self.strategy_bound_handle();
        // 先发送到strategy_output_handle，确保事件一定会被发送
        let order_event =
            self.generate_order_node_event(strategy_output_handle.output_handle_id().clone(), virtual_order.clone(), event_type);
        if let Some(order_event) = order_event {
            self.strategy_bound_handle_send(order_event.into())?;
        }

        if self.is_leaf_node() {
            self.send_execute_over_event(Some(virtual_order.order_config_id), Some(self.strategy_time()))?;
            Ok(())
        } else {
            let order_status_output_handle_id =
                format!("{}_{}_output_{}", node_id, order_status.to_string(), virtual_order.order_config_id);
            let order_status_output_handle = self.output_handle(&order_status_output_handle_id)?;

            let order_event = self.generate_order_node_event(
                order_status_output_handle.output_handle_id().clone(),
                virtual_order.clone(),
                event_type,
            );
            if let Some(order_event) = order_event {
                order_status_output_handle.send(order_event.into())?;
            }
            // 订单配置id对应的all_output_handle_id
            let order_all_status_output_handle_id = format!("{}_all_status_output_{}", node_id, virtual_order.order_config_id);
            let order_all_status_output_handle = self.output_handle(&order_all_status_output_handle_id)?;
            let order_event = self.generate_order_node_event(
                order_all_status_output_handle.output_handle_id().clone(),
                virtual_order.clone(),
                event_type,
            );
            if let Some(order_event) = order_event {
                order_all_status_output_handle.send(order_event.into())?;
            }
            Ok(())
        }
    }

    // 处理虚拟交易系统事件
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
                let input_handle_id = format!("{}_input_{}", self.node_id(), order.order_config_id);
                match virtual_trading_system_event {
                    VtsEvent::FuturesOrderCreated(_) => {
                        tracing::debug!("[{}] receive futures order created event", self.node_name());
                        self.add_unfilled_virtual_order(&input_handle_id, order.clone()).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderCreatedMsg::new(
                            order.order_id,
                            order.order_config_id,
                            order.open_price,
                            order.order_side.to_string(),
                        );
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCreated,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.create_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    VtsEvent::FuturesOrderFilled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderFilledMsg::new(self.node_name().clone(), order.order_id, order.quantity, order.open_price);
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderFilled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    VtsEvent::FuturesOrderCanceled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await?;
                        let message = OrderCanceledMsg::new(self.node_name().clone(), order.order_id);
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
                            self.cycle_id(),
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCanceled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        )
                        .into();
                        self.strategy_bound_handle_send(log_event.into())?;
                    }

                    // 只是发送事件
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
                self.add_virtual_transaction_history(&input_handle_id, transaction.clone()).await;
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
                let _ = self.strategy_bound_handle_send(transaction_event.into());
            }
        }

        Ok(())
    }
}
