use super::super::super::node_message::futures_order_node_log_message::*;
use super::FuturesOrderNodeContext;
use crate::backtest_strategy_engine::node::node_context::BacktestNodeContextTrait;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::GetCurrentTimeCommand;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};
use event_center::event::node_event::backtest_node_event::futures_order_node_event::*;
use event_center::event::node_event::backtest_node_event::if_else_node_event::IfElseNodeEvent;
use event_center::event::strategy_event::{StrategyRunningLogEvent, StrategyRunningLogSource, StrategyRunningLogType};
use star_river_core::custom_type::InputHandleId;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::futures_order_node_error::*;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::system::DateTimeUtc;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use star_river_core::virtual_trading_system::event::VirtualTradingSystemEvent;
use tokio::sync::oneshot;

impl FuturesOrderNodeContext {
    /// 使用第一个有连接的output_handle发送trigger事件
    async fn send_trigger_event_spec(&self) {
        if self.is_leaf_node() {
            self.send_execute_over_event().await;
            return;
        }
        let all_output_handles = self.get_all_output_handles();
        let strategy_output_handle_id = format!("{}_strategy_output", self.get_node_id());

        // 使用filter过滤出连接数大于0的handle，然后取第一个
        if let Some((handle_id, handle)) = all_output_handles
            .iter()
            .filter(|(handle_id, handle)| handle_id != &&strategy_output_handle_id && handle.connect_count > 0)
            .next()
        {
            let payload = TriggerPayload::new(self.get_play_index());
            let trigger_event = TriggerEvent::new(self.get_node_id().clone(), self.get_node_name().clone(), handle_id.clone(), payload);
            let _ = handle.send(BacktestNodeEvent::Common(trigger_event.into()));
        } else {
            return;
        }
    }

    pub async fn handle_node_event_for_specific_order(
        &mut self,
        node_event: BacktestNodeEvent,
        input_handle_id: &InputHandleId,
    ) -> Result<(), FuturesOrderNodeError> {
        tracing::debug!("{}: 接收器 {} 接收到节点事件: {:?}", self.get_node_id(), input_handle_id, node_event);
        match node_event {
            BacktestNodeEvent::Common(common_evt) => match common_evt {
                CommonEvent::Trigger(trigger_evt) => {
                    if trigger_evt.play_index == self.get_play_index() {
                        self.send_trigger_event_spec().await;
                    }
                }

                _ => {}
            },
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(condition_match_evt)) => {
                if condition_match_evt.play_index == self.get_play_index() {
                    // 根据input_handle_id获取订单配置
                    let order_config = {
                        self.node_config
                            .futures_order_configs
                            .iter()
                            .find(|config| config.input_handle_id == *input_handle_id)
                            .ok_or(
                                OrderConfigNotFoundSnafu {
                                    input_handle_id: input_handle_id.to_string(),
                                }
                                .build(),
                            )?
                            .clone()
                    };
                    // 创建订单
                    let create_order_result = self.create_order(&order_config).await;
                    if let Err(e) = create_order_result {
                        // 发送trigger事件
                        self.send_trigger_event_spec().await;
                        return Err(e);
                    }
                } else {
                    tracing::warn!("{}: 当前k线缓存索引不匹配, 跳过", self.get_node_id());
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) async fn send_order_status_event(&mut self, virtual_order: VirtualOrder, event_type: &VirtualTradingSystemEvent) {
        let order_status = &virtual_order.order_status;
        let node_id = self.get_node_id().clone();
        let order_status_output_handle_id = format!("{}_{}_output_{}", node_id, order_status.to_string(), virtual_order.order_config_id);
        // 订单配置id对应的all_output_handle_id
        let config_all_status_output_handle_id = format!("{}_all_status_output_{}", node_id, virtual_order.order_config_id);
        tracing::debug!("output_handle_id: {}", order_status_output_handle_id);

        let order_status_output_handle = self.get_output_handle(&order_status_output_handle_id);
        let config_all_status_output_handle = self.get_output_handle(&config_all_status_output_handle_id);

        // 如果总输出与订单状态输出都没有连接，则发送trigger事件
        if config_all_status_output_handle.connect_count == 0 && order_status_output_handle.connect_count == 0 {
            tracing::debug!("all_status_output_handle and order_status_output_handle connect_count are 0, send trigger event");
            self.send_trigger_event_spec().await;
            return;
        }

        // 如果all_status_output_handle的connect_count大于0，则发送事件
        if config_all_status_output_handle.connect_count > 0 {
            let order_event = self.get_order_node_event(config_all_status_output_handle_id.clone(), virtual_order.clone(), event_type);
            if let Some(order_event) = order_event {
                tracing::debug!(
                    "send order event through all_status_output_handle: {}",
                    config_all_status_output_handle_id
                );
                let _ = config_all_status_output_handle.send(order_event.into());
            }
        }
        if order_status_output_handle.connect_count > 0 {
            let order_event = self.get_order_node_event(order_status_output_handle_id.clone(), virtual_order.clone(), event_type);
            if let Some(order_event) = order_event {
                tracing::debug!(
                    "send order event through order_status_output_handle: {}",
                    order_status_output_handle_id
                );
                let _ = order_status_output_handle.send(order_event.into());
            }
        }

        let order_event = self.get_order_node_event(config_all_status_output_handle_id.clone(), virtual_order.clone(), event_type);
        let strategy_output_handle = self.get_strategy_output_handle();
        if let Some(order_event) = order_event {
            tracing::debug!(
                "send order event through strategy_output_handle: {}",
                config_all_status_output_handle_id
            );
            let _ = strategy_output_handle.send(order_event.into());
        }
    }

    fn get_order_node_event(
        &self,
        output_handle_id: String,
        virtual_order: VirtualOrder,
        event_type: &VirtualTradingSystemEvent,
    ) -> Option<FuturesOrderNodeEvent> {
        let node_id = self.get_node_id().clone();
        let node_name = self.get_node_name().clone();

        let order_event: Option<FuturesOrderNodeEvent> = match event_type {
            // 期货订单事件
            VirtualTradingSystemEvent::FuturesOrderCreated(_) => {
                let payload = FuturesOrderCreatedPayload::new(virtual_order.clone());
                Some(FuturesOrderCreatedEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::FuturesOrderFilled(_) => {
                let payload = FuturesOrderFilledPayload::new(virtual_order.clone());
                Some(FuturesOrderFilledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::FuturesOrderCanceled(_) => {
                let payload = FuturesOrderCanceledPayload::new(virtual_order.clone());
                Some(FuturesOrderCanceledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }

            // 止盈订单事件
            VirtualTradingSystemEvent::TakeProfitOrderCreated(_) => {
                let payload = TakeProfitOrderCreatedPayload::new(virtual_order.clone());
                Some(TakeProfitOrderCreatedEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::TakeProfitOrderFilled(_) => {
                let payload = TakeProfitOrderFilledPayload::new(virtual_order.clone());
                Some(TakeProfitOrderFilledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::TakeProfitOrderCanceled(_) => {
                let payload = TakeProfitOrderCanceledPayload::new(virtual_order.clone());
                Some(TakeProfitOrderCanceledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }

            // 止损订单事件
            VirtualTradingSystemEvent::StopLossOrderCreated(_) => {
                let payload = StopLossOrderCreatedPayload::new(virtual_order.clone());
                Some(StopLossOrderCreatedEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::StopLossOrderFilled(_) => {
                let payload = StopLossOrderFilledPayload::new(virtual_order.clone());
                Some(StopLossOrderFilledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            VirtualTradingSystemEvent::StopLossOrderCanceled(_) => {
                let payload = StopLossOrderCanceledPayload::new(virtual_order.clone());
                Some(StopLossOrderCanceledEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into())
            }
            _ => None,
        };
        order_event
    }

    // 处理虚拟交易系统事件
    pub(crate) async fn handle_virtual_trading_system_event(
        &mut self,
        virtual_trading_system_event: VirtualTradingSystemEvent,
    ) -> Result<(), String> {
        let order: Option<&VirtualOrder> = match &virtual_trading_system_event {
            VirtualTradingSystemEvent::FuturesOrderCreated(order)
            | VirtualTradingSystemEvent::FuturesOrderFilled(order)
            | VirtualTradingSystemEvent::FuturesOrderCanceled(order)
            | VirtualTradingSystemEvent::TakeProfitOrderCreated(order)
            | VirtualTradingSystemEvent::TakeProfitOrderFilled(order)
            | VirtualTradingSystemEvent::TakeProfitOrderCanceled(order)
            | VirtualTradingSystemEvent::StopLossOrderCreated(order)
            | VirtualTradingSystemEvent::StopLossOrderFilled(order)
            | VirtualTradingSystemEvent::StopLossOrderCanceled(order) => Some(order),
            _ => None,
        };

        if let Some(order) = order {
            if order.node_id == self.get_node_id().clone() {
                let input_handle_id = format!("{}_input_{}", self.get_node_id(), order.order_config_id);
                match virtual_trading_system_event {
                    VirtualTradingSystemEvent::FuturesOrderCreated(_) => {
                        tracing::debug!("[{}] receive futures order created event", self.get_node_name());
                        self.add_unfilled_virtual_order(&input_handle_id, order.clone()).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCreatedMsg::new(
                            order.order_id,
                            order.order_config_id,
                            order.open_price,
                            order.order_side.to_string(),
                        );
                        let log_event = StrategyRunningLogEvent::success(
                            self.get_strategy_id().clone(),
                            self.get_node_id().clone(),
                            self.get_node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCreated,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.create_time,
                        );
                        let _ = self.get_strategy_output_handle().send(log_event.into());
                    }

                    VirtualTradingSystemEvent::FuturesOrderFilled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderFilledMsg::new(self.get_node_name().clone(), order.order_id, order.quantity, order.open_price);
                        let log_event = StrategyRunningLogEvent::success(
                            self.get_strategy_id().clone(),
                            self.get_node_id().clone(),
                            self.get_node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderFilled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        );
                        let _ = self.get_strategy_output_handle().send(log_event.into());
                    }

                    VirtualTradingSystemEvent::FuturesOrderCanceled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCanceledMsg::new(self.get_node_name().clone(), order.order_id);
                        let log_event = StrategyRunningLogEvent::success(
                            self.get_strategy_id().clone(),
                            self.get_node_id().clone(),
                            self.get_node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCanceled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        );
                        let _ = self.get_strategy_output_handle().send(log_event.into());
                    }

                    // 只是发送事件
                    VirtualTradingSystemEvent::TakeProfitOrderCreated(_)
                    | VirtualTradingSystemEvent::TakeProfitOrderFilled(_)
                    | VirtualTradingSystemEvent::TakeProfitOrderCanceled(_)
                    | VirtualTradingSystemEvent::StopLossOrderCreated(_)
                    | VirtualTradingSystemEvent::StopLossOrderFilled(_)
                    | VirtualTradingSystemEvent::StopLossOrderCanceled(_) => {
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                    }

                    _ => {}
                }
            }
        }

        let transaction: Option<&VirtualTransaction> = match &virtual_trading_system_event {
            VirtualTradingSystemEvent::TransactionCreated(transaction) => Some(transaction),
            _ => None,
        };

        if let Some(transaction) = transaction {
            if transaction.node_id == self.get_node_id().clone() {
                let input_handle_id = format!("{}_input_{}", self.get_node_id(), transaction.order_config_id);
                self.add_virtual_transaction_history(&input_handle_id, transaction.clone()).await;
                let payload = TransactionCreatedPayload::new(transaction.clone());
                let transaction_event: FuturesOrderNodeEvent = TransactionCreatedEvent::new(
                    self.get_node_id().clone(),
                    self.get_node_name().clone(),
                    input_handle_id.clone(),
                    payload,
                )
                .into();
                let strategy_output_handle = self.get_strategy_output_handle();
                let _ = strategy_output_handle.send(transaction_event.into());
            }
        }

        Ok(())
    }

    pub(super) async fn get_current_time(&self) -> Result<DateTimeUtc, String> {
        let (tx, rx) = oneshot::channel();
        let cmd = GetCurrentTimeCommand::new(self.get_node_id().clone(), tx, None);
        self.get_strategy_command_sender().send(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.current_time.clone());
        } else {
            return Err("获取当前时间失败".to_string());
        }
    }
}
