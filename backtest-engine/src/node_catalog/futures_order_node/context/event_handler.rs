use async_trait::async_trait;
use event_center::Event;
use star_river_core::custom_type::InputHandleId;
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
    node_error::futures_order_node_error::OrderConfigNotFoundSnafu,
    node_event::BacktestNodeEvent,
    node_message::futures_order_node_log_message::{OrderCanceledMsg, OrderCreatedMsg, OrderFilledMsg},
};

#[async_trait]
impl NodeEventHandlerExt for FuturesOrderNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) {}

    async fn handle_source_node_event(&mut self, _node_event: BacktestNodeEvent) {}

    async fn handle_command(&mut self, node_command: BacktestNodeCommand) {
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
                }
            }
            BacktestNodeCommand::GetFuturesOrderConfig(cmd) => {
                tracing::debug!("@[{}] received get futures order config command", self.node_name());
                if cmd.node_id() == self.node_id() {
                    let futures_order_node_config = self.node_config.clone();
                    let payload = GetFuturesOrderConfigRespPayload::new(futures_order_node_config);
                    let response = GetFuturesOrderConfigResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}

impl FuturesOrderNodeContext {
    /// 使用第一个有连接的output_handle发送trigger事件

    pub async fn handle_node_event_for_independent_order(&mut self, node_event: BacktestNodeEvent, order_config_id: i32) {
        match node_event {
            BacktestNodeEvent::Common(common_evt) => match common_evt {
                CommonEvent::Trigger(_trigger_evt) => {
                    let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
                    let phase_name = format!("handle trigger event for specific order");
                    cycle_tracker.start_phase(&phase_name);

                    if self.is_leaf_node() {
                        self.send_execute_over_event(self.play_index() as u64, None).unwrap();
                        return;
                    } else {
                        self.independent_order_send_trigger_event(order_config_id).await;
                    }

                    cycle_tracker.end_phase(&phase_name);
                    let completed_tracker = cycle_tracker.end();
                    self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                        .await
                        .unwrap();
                }

                _ => {}
            },
            BacktestNodeEvent::IfElseNode(ifelse_node_event) => {
                match ifelse_node_event {
                    IfElseNodeEvent::CaseTrue(_) | IfElseNodeEvent::ElseTrue(_) => {
                        let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);

                        let phase_name = format!("handle condition match event for order {}", order_config_id);
                        cycle_tracker.start_phase(&phase_name);

                        // 根据input_handle_id获取订单配置
                        let order_config = {
                            self.node_config
                                .futures_order_configs
                                .iter()
                                .find(|config| config.order_config_id == order_config_id)
                                .ok_or(
                                    OrderConfigNotFoundSnafu {
                                        order_config_id: order_config_id,
                                    }
                                    .build(),
                                )
                                .unwrap()
                                .clone()
                        };

                        // 创建订单
                        let create_order_result = self.create_order(&order_config).await;
                        if let Err(_) = create_order_result {
                            // 发送trigger事件
                            self.independent_order_send_trigger_event(order_config_id).await;
                            cycle_tracker.end_phase(&phase_name);
                            let completed_tracker = cycle_tracker.end();
                            self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                                .await
                                .unwrap();
                            return;
                        }
                        cycle_tracker.end_phase(&phase_name);
                        let completed_tracker = cycle_tracker.end();
                        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
                            .await
                            .unwrap();
                    }
                    IfElseNodeEvent::CaseFalse(_) | IfElseNodeEvent::ElseFalse(_) => {
                        tracing::debug!("@[{}] receive event {}", self.node_name(), order_config_id);
                        if self.is_leaf_node() {
                            self.send_execute_over_event(self.play_index() as u64, Some(order_config_id))
                                .unwrap();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) async fn send_order_status_event(&mut self, virtual_order: VirtualOrder, event_type: &VtsEvent) {
        let order_status = &virtual_order.order_status;
        let node_id = self.node_id().clone();
        let order_status_output_handle_id = format!("{}_{}_output_{}", node_id, order_status.to_string(), virtual_order.order_config_id);
        // 订单配置id对应的all_output_handle_id
        let config_all_status_output_handle_id = format!("{}_all_status_output_{}", node_id, virtual_order.order_config_id);
        tracing::debug!("output_handle_id: {}", order_status_output_handle_id);

        let order_status_output_handle = self.output_handle(&order_status_output_handle_id).unwrap();
        let config_all_status_output_handle = self.output_handle(&config_all_status_output_handle_id).unwrap();

        // 先发送到strategy_output_handle，确保事件一定会被发送
        let order_event = self.genarate_order_node_event(config_all_status_output_handle_id.clone(), virtual_order.clone(), event_type);
        let strategy_output_handle = self.strategy_bound_handle();
        if let Some(order_event) = order_event {
            tracing::debug!(
                "send order event through strategy_output_handle: {}",
                config_all_status_output_handle_id
            );
            let _ = strategy_output_handle.send(order_event.into());
        }

        // 如果总输出与订单状态输出都没有连接，则发送trigger事件
        if !config_all_status_output_handle.is_connected() && !order_status_output_handle.is_connected() {
            tracing::debug!("all_status_output_handle and order_status_output_handle connect_count are 0, send trigger event");
            self.independent_order_send_trigger_event(virtual_order.order_config_id).await;
            return;
        }

        // 如果all_status_output_handle的connect_count大于0，则发送事件
        if config_all_status_output_handle.is_connected() {
            let order_event = self.genarate_order_node_event(config_all_status_output_handle_id.clone(), virtual_order.clone(), event_type);
            if let Some(order_event) = order_event {
                tracing::debug!(
                    "send order event through all_status_output_handle: {}",
                    config_all_status_output_handle_id
                );
                let _ = config_all_status_output_handle.send(order_event.into());
            }
        }
        if order_status_output_handle.is_connected() {
            let order_event = self.genarate_order_node_event(order_status_output_handle_id.clone(), virtual_order.clone(), event_type);
            if let Some(order_event) = order_event {
                tracing::debug!(
                    "send order event through order_status_output_handle: {}",
                    order_status_output_handle_id
                );
                let _ = order_status_output_handle.send(order_event.into());
            }
        }
    }

    // 处理虚拟交易系统事件
    pub(crate) async fn handle_vts_event(&mut self, virtual_trading_system_event: VtsEvent) -> Result<(), String> {
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
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCreatedMsg::new(
                            order.order_id,
                            order.order_config_id,
                            order.open_price,
                            order.order_side.to_string(),
                        );
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
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
                        let _ = self.strategy_bound_handle_send(log_event.into());
                    }

                    VtsEvent::FuturesOrderFilled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderFilledMsg::new(self.node_name().clone(), order.order_id, order.quantity, order.open_price);
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
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
                        let _ = self.strategy_bound_handle_send(log_event.into());
                    }

                    VtsEvent::FuturesOrderCanceled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCanceledMsg::new(self.node_name().clone(), order.order_id);
                        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
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
                        let _ = self.strategy_bound_handle_send(log_event.into());
                    }

                    // 只是发送事件
                    VtsEvent::TakeProfitOrderCreated(_)
                    | VtsEvent::TakeProfitOrderFilled(_)
                    | VtsEvent::TakeProfitOrderCanceled(_)
                    | VtsEvent::StopLossOrderCreated(_)
                    | VtsEvent::StopLossOrderFilled(_)
                    | VtsEvent::StopLossOrderCanceled(_) => {
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
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
                let transaction_event: FuturesOrderNodeEvent =
                    TransactionCreatedEvent::new(self.node_id().clone(), self.node_name().clone(), input_handle_id.clone(), payload).into();
                let _ = self.strategy_bound_handle_send(transaction_event.into());
            }
        }

        Ok(())
    }
}
