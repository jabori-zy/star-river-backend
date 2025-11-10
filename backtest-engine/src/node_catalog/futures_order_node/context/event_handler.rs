use super::FuturesOrderNodeContext;
use strategy_core::node::context_trait::{NodeEventHandlerExt, NodeHandleExt};
use async_trait::async_trait;
use event_center::Event;
use crate::node::node_event::BacktestNodeEvent;
use crate::node::node_command::{BacktestNodeCommand, NodeResetRespPayload};
use strategy_core::node::context_trait::NodeIdentityExt;
use crate::node::node_command::NodeResetResponse;
use strategy_core::node::context_trait::NodeRelationExt;
use strategy_core::node::context_trait::NodeCommunicationExt;
use star_river_core::custom_type::InputHandleId;
use strategy_core::event::node_common_event::TriggerPayload;
use strategy_core::event::node_common_event::TriggerEvent;
use strategy_core::event::node_common_event::CommonEvent;
use strategy_core::benchmark::node_benchmark::CycleTracker;
use virtual_trading::types::VirtualOrder;
use star_river_event::backtest_strategy::node_event::futures_order_node_event::{
    TakeProfitOrderCanceledPayload,
    TakeProfitOrderCreatedPayload,
    TakeProfitOrderFilledPayload,
    StopLossOrderCanceledPayload,
    StopLossOrderCreatedPayload,
    StopLossOrderFilledPayload,
    FuturesOrderCanceledPayload,
    FuturesOrderCreatedPayload,
    FuturesOrderFilledPayload,
    FuturesOrderCreatedEvent,
    FuturesOrderFilledEvent,
    FuturesOrderCanceledEvent,
    TakeProfitOrderCreatedEvent,
    TakeProfitOrderFilledEvent,
    TakeProfitOrderCanceledEvent,
    StopLossOrderCreatedEvent,
    StopLossOrderFilledEvent,
    StopLossOrderCanceledEvent,
};
use crate::node::node_error::FuturesOrderNodeError;
use strategy_core::node::context_trait::NodeBenchmarkExt;
use star_river_event::backtest_strategy::node_event::IfElseNodeEvent;
use crate::node::node_error::futures_order_node_error::OrderConfigNotFoundSnafu;
use star_river_event::backtest_strategy::node_event::FuturesOrderNodeEvent;
use virtual_trading::event::VirtualTradingSystemEvent;
use strategy_core::event::log_event::StrategyRunningLogEvent;
use crate::node::node_message::futures_order_node_log_message::OrderCreatedMsg;
use strategy_core::event::log_event::StrategyRunningLogSource;
use strategy_core::event::log_event::StrategyRunningLogType;
use crate::node::node_message::futures_order_node_log_message::OrderCanceledMsg;
use crate::node::node_message::futures_order_node_log_message::OrderFilledMsg;
use star_river_event::backtest_strategy::node_event::futures_order_node_event::TransactionCreatedEvent;
use star_river_event::backtest_strategy::node_event::futures_order_node_event::TransactionCreatedPayload;
use tokio::sync::oneshot;
use star_river_core::system::DateTimeUtc;
use virtual_trading::types::VirtualTransaction;
use crate::strategy::strategy_command::{GetCurrentTimeCmdPayload, GetCurrentTimeCommand};
use strategy_core::communication::strategy::StrategyResponse;


#[async_trait]
impl NodeEventHandlerExt for FuturesOrderNodeContext {
    type EngineEvent = Event;

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        tracing::info!("[{}] received node event: {:?}", self.node_name(), node_event);
    }
    
    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
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
            _ => {}
        }
    }
}



impl FuturesOrderNodeContext {
    /// 使用第一个有连接的output_handle发送trigger事件
    async fn send_trigger_event_spec(&self) {
        if self.is_leaf_node() {
            self.send_execute_over_event();
            return;
        }
        let all_output_handles = self.output_handles();
        let strategy_output_handle_id = format!("{}_strategy_output", self.node_id());

        // 使用filter过滤出连接数大于0的handle，然后取第一个
        if let Some((handle_id, handle)) = all_output_handles
            .iter()
            .filter(|(handle_id, handle)| handle_id != &&strategy_output_handle_id && handle.is_connected())
            .next()
        {
            let payload = TriggerPayload::new(self.play_index() as u64);
            let trigger_event = TriggerEvent::new(self.node_id().clone(), self.node_name().clone(), handle_id.clone(), payload);
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
        // tracing::debug!("{}: 接收器 {} 接收到节点事件: {:?}", self.node_id(), input_handle_id, node_event);
        match node_event {
            BacktestNodeEvent::Common(common_evt) => match common_evt {
                CommonEvent::Trigger(trigger_evt) => {
                    let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
                    let phase_name = format!("handle trigger event for specific order");
                    cycle_tracker.start_phase(&phase_name);
                    if trigger_evt.cycle_id == self.play_index() as u64 {
                        self.send_trigger_event_spec().await;
                    }
                    cycle_tracker.end_phase(&phase_name);
                    let completed_tracker = cycle_tracker.end();
                    self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
                }

                _ => {}
            },
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(condition_match_evt)) => {
                if condition_match_evt.play_index == self.play_index() {
                    let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);
                    // 根据input_handle_id获取订单配置
                    //"if_else_node_1761319649542_p2q1x2e_output_1"
                    // 取最后一个_后面的数字
                    let order_config = input_handle_id.split("_").last().unwrap();
                    let order_config_id = order_config.parse::<i32>().unwrap();
                    let phase_name = format!("handle condition match event for order {}", order_config_id);
                    cycle_tracker.start_phase(&phase_name);
                    
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
                        cycle_tracker.end_phase(&phase_name);
                        let completed_tracker = cycle_tracker.end();
                        self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
                        return Err(e);
                    }
                    cycle_tracker.end_phase(&phase_name);
                    let completed_tracker = cycle_tracker.end();
                    self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
                } else {
                    tracing::warn!("{}: 当前k线缓存索引不匹配, 跳过", self.node_id());
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) async fn send_order_status_event(&mut self, virtual_order: VirtualOrder, event_type: &VirtualTradingSystemEvent) {
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
            self.send_trigger_event_spec().await;
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

    
    fn genarate_order_node_event(
        &self,
        output_handle_id: String,
        virtual_order: VirtualOrder,
        event_type: &VirtualTradingSystemEvent,
    ) -> Option<FuturesOrderNodeEvent> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

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
            if order.node_id == self.node_id().clone() {
                let input_handle_id = format!("{}_input_{}", self.node_id(), order.order_config_id);
                match virtual_trading_system_event {
                    VirtualTradingSystemEvent::FuturesOrderCreated(_) => {
                        tracing::debug!("[{}] receive futures order created event", self.node_name());
                        self.add_unfilled_virtual_order(&input_handle_id, order.clone()).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCreatedMsg::new(
                            order.order_id,
                            order.order_config_id,
                            order.open_price,
                            order.order_side.to_string(),
                        );
                        let log_event: CommonEvent = StrategyRunningLogEvent::success(
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCreated,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.create_time,
                        ).into();
                        let _ = self.strategy_bound_handle_send(log_event.into());
                    }

                    VirtualTradingSystemEvent::FuturesOrderFilled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderFilledMsg::new(self.node_name().clone(), order.order_id, order.quantity, order.open_price);
                        let log_event: CommonEvent = StrategyRunningLogEvent::success(
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderFilled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        ).into();
                        let _ = self.strategy_bound_handle_send(log_event.into());
                    }

                    VirtualTradingSystemEvent::FuturesOrderCanceled(_) => {
                        self.remove_unfilled_virtual_order(&input_handle_id, order.order_id).await;
                        self.add_virtual_order_history(&input_handle_id, order.clone()).await;
                        self.set_is_processing_order(&input_handle_id, false).await;
                        self.send_order_status_event(order.clone(), &virtual_trading_system_event).await;
                        let message = OrderCanceledMsg::new(self.node_name().clone(), order.order_id);
                        let log_event: CommonEvent = StrategyRunningLogEvent::success(
                            self.strategy_id().clone(),
                            self.node_id().clone(),
                            self.node_name().clone(),
                            StrategyRunningLogSource::Node,
                            StrategyRunningLogType::OrderCanceled,
                            message.to_string(),
                            serde_json::to_value(order).unwrap(),
                            order.update_time,
                        ).into();
                        let _ = self.strategy_bound_handle_send(log_event.into());
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
            if transaction.node_id == self.node_id().clone() {
                let input_handle_id = format!("{}_input_{}", self.node_id(), transaction.order_config_id);
                self.add_virtual_transaction_history(&input_handle_id, transaction.clone()).await;
                let payload = TransactionCreatedPayload::new(transaction.clone());
                let transaction_event: FuturesOrderNodeEvent = TransactionCreatedEvent::new(
                    self.node_id().clone(),
                    self.node_name().clone(),
                    input_handle_id.clone(),
                    payload,
                )
                .into();
                let _ = self.strategy_bound_handle_send(transaction_event.into());

                
            }
        }

        Ok(())
    }

    pub(super) async fn get_current_time(&self) -> Result<DateTimeUtc, String> {
        let (tx, rx) = oneshot::channel();
        let payload = GetCurrentTimeCmdPayload;
        let cmd = GetCurrentTimeCommand::new(self.node_id().clone(), tx, payload);
        self.send_strategy_command(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload,.. } => {
                return Ok(payload.current_time.clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(error.to_string());
            }
        }
    }
}
