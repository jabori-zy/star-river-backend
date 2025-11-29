use star_river_core::custom_type::HandleId;
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
use strategy_core::node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt};
use virtual_trading::{event::VtsEvent, types::VirtualOrder};

use super::FuturesOrderNodeContext;
use crate::node::node_error::FuturesOrderNodeError;

impl FuturesOrderNodeContext {
    pub(super) async fn independent_order_send_trigger_event(&self, config_id: i32) -> Result<(), FuturesOrderNodeError> {
        let all_output_handles = self.output_handles();
        let futures = all_output_handles
            .values()
            .filter(|handle| handle.config_id() == config_id)
            .map(|handle| self.send_trigger_event(handle.output_handle_id(), Some(self.strategy_time())));

        futures::future::try_join_all(futures).await?;
        Ok(())
    }

    pub(super) fn generate_order_node_event(
        &self,
        output_handle_id: HandleId,
        virtual_order: VirtualOrder,
        vts_event: &VtsEvent,
    ) -> Option<FuturesOrderNodeEvent> {
        macro_rules! create_event {
            ($event_type:ident, $payload_type:ident) => {{
                let payload = $payload_type::new(virtual_order);
                Some(
                    $event_type::new_with_time(
                        self.cycle_id(),
                        self.node_id().clone(),
                        self.node_name().clone(),
                        output_handle_id,
                        self.strategy_time(),
                        payload,
                    )
                    .into(),
                )
            }};
        }

        match vts_event {
            // 期货订单事件
            VtsEvent::FuturesOrderCreated(_) => create_event!(FuturesOrderCreatedEvent, FuturesOrderCreatedPayload),
            VtsEvent::FuturesOrderFilled(_) => create_event!(FuturesOrderFilledEvent, FuturesOrderFilledPayload),
            VtsEvent::FuturesOrderCanceled(_) => create_event!(FuturesOrderCanceledEvent, FuturesOrderCanceledPayload),
            // 止盈订单事件
            VtsEvent::TakeProfitOrderCreated(_) => create_event!(TakeProfitOrderCreatedEvent, TakeProfitOrderCreatedPayload),
            VtsEvent::TakeProfitOrderFilled(_) => create_event!(TakeProfitOrderFilledEvent, TakeProfitOrderFilledPayload),
            VtsEvent::TakeProfitOrderCanceled(_) => create_event!(TakeProfitOrderCanceledEvent, TakeProfitOrderCanceledPayload),
            // 止损订单事件
            VtsEvent::StopLossOrderCreated(_) => create_event!(StopLossOrderCreatedEvent, StopLossOrderCreatedPayload),
            VtsEvent::StopLossOrderFilled(_) => create_event!(StopLossOrderFilledEvent, StopLossOrderFilledPayload),
            VtsEvent::StopLossOrderCanceled(_) => create_event!(StopLossOrderCanceledEvent, StopLossOrderCanceledPayload),
            _ => None,
        }
    }
}
