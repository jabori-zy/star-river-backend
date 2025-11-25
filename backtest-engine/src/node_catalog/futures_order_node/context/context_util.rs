use star_river_core::system::DateTimeUtc;
use star_river_event::backtest_strategy::node_event::{
    FuturesOrderNodeEvent,
    futures_order_node_event::{
        FuturesOrderCanceledEvent, FuturesOrderCanceledPayload, FuturesOrderCreatedEvent, FuturesOrderCreatedPayload,
        FuturesOrderFilledEvent, FuturesOrderFilledPayload, StopLossOrderCanceledEvent, StopLossOrderCanceledPayload,
        StopLossOrderCreatedEvent, StopLossOrderCreatedPayload, StopLossOrderFilledEvent, StopLossOrderFilledPayload,
        TakeProfitOrderCanceledEvent, TakeProfitOrderCanceledPayload, TakeProfitOrderCreatedEvent, TakeProfitOrderCreatedPayload,
        TakeProfitOrderFilledEvent, TakeProfitOrderFilledPayload,
    },
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use tokio::sync::oneshot;
use virtual_trading::{event::VtsEvent, types::VirtualOrder};

use super::FuturesOrderNodeContext;

impl FuturesOrderNodeContext {
    pub(super) async fn independent_order_send_trigger_event(&self, config_id: i32) {
        let all_output_handles = self.output_handles();
        tracing::debug!("send trigger event to order output handles: {:#?}", all_output_handles);
        let futures = all_output_handles
            .values()
            .filter(|handle| handle.config_id() == config_id)
            .map(|handle| self.send_trigger_event(handle.output_handle_id(), Some(self.strategy_time())));

        futures::future::join_all(futures).await;
    }

    pub(super) fn genarate_order_node_event(
        &self,
        output_handle_id: String,
        virtual_order: VirtualOrder,
        event_type: &VtsEvent,
    ) -> Option<FuturesOrderNodeEvent> {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();

        let order_event: Option<FuturesOrderNodeEvent> = match event_type {
            // 期货订单事件
            VtsEvent::FuturesOrderCreated(_) => {
                let payload = FuturesOrderCreatedPayload::new(virtual_order.clone());
                Some(
                    FuturesOrderCreatedEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::FuturesOrderFilled(_) => {
                let payload = FuturesOrderFilledPayload::new(virtual_order.clone());
                Some(
                    FuturesOrderFilledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::FuturesOrderCanceled(_) => {
                let payload = FuturesOrderCanceledPayload::new(virtual_order.clone());
                Some(
                    FuturesOrderCanceledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }

            // 止盈订单事件
            VtsEvent::TakeProfitOrderCreated(_) => {
                let payload = TakeProfitOrderCreatedPayload::new(virtual_order.clone());
                Some(
                    TakeProfitOrderCreatedEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::TakeProfitOrderFilled(_) => {
                let payload = TakeProfitOrderFilledPayload::new(virtual_order.clone());
                Some(
                    TakeProfitOrderFilledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::TakeProfitOrderCanceled(_) => {
                let payload = TakeProfitOrderCanceledPayload::new(virtual_order.clone());
                Some(
                    TakeProfitOrderCanceledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }

            // 止损订单事件
            VtsEvent::StopLossOrderCreated(_) => {
                let payload = StopLossOrderCreatedPayload::new(virtual_order.clone());
                Some(
                    StopLossOrderCreatedEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::StopLossOrderFilled(_) => {
                let payload = StopLossOrderFilledPayload::new(virtual_order.clone());
                Some(
                    StopLossOrderFilledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            VtsEvent::StopLossOrderCanceled(_) => {
                let payload = StopLossOrderCanceledPayload::new(virtual_order.clone());
                Some(
                    StopLossOrderCanceledEvent::new_with_time(
                        self.cycle_id(),
                        node_id.clone(),
                        node_name.clone(),
                        output_handle_id.clone(),
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }
            _ => None,
        };
        order_event
    }
}
