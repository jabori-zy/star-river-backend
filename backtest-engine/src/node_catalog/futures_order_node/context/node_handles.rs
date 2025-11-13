use star_river_core::order::OrderType;
use strategy_core::node::context_trait::{NodeHandleExt, NodeIdentityExt};
use tokio::sync::broadcast;

use super::FuturesOrderNodeContext;
use crate::node::node_event::BacktestNodeEvent;

impl NodeHandleExt for FuturesOrderNodeContext {
    fn set_output_handles(&mut self) {
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let futures_order_configs = self.node_config.futures_order_configs.clone();

        // 为每一个订单添加出口
        for order_config in futures_order_configs.iter() {
            let all_output_handle_id = format!("{}_all_status_output_{}", node_id, order_config.order_config_id);
            let (all_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order all output handle: {}", all_output_handle_id);
            self.add_output_handle(false, all_output_handle_id, all_tx);

            let created_output_handle_id = format!("{}_created_output_{}", node_id, order_config.order_config_id);
            let (created_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order created output handle: {}", created_output_handle_id);
            self.add_output_handle(false, created_output_handle_id, created_tx);

            match order_config.order_type {
                OrderType::Limit => {
                    let placed_output_handle_id = format!("{}_placed_output_{}", node_id, order_config.order_config_id);
                    let (placed_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
                    tracing::debug!("[{node_name}] setting order placed output handle: {}", placed_output_handle_id);
                    self.add_output_handle(false, placed_output_handle_id, placed_tx);
                }
                _ => {}
            }

            let partial_output_handle_id = format!("{}_partial_output_{}", node_id, order_config.order_config_id);
            let (partial_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order partial output handle: {}", partial_output_handle_id);
            self.add_output_handle(false, partial_output_handle_id, partial_tx);

            let filled_output_handle_id = format!("{}_filled_output_{}", node_id, order_config.order_config_id);
            let (filled_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order filled output handle: {}", filled_output_handle_id);
            self.add_output_handle(false, filled_output_handle_id, filled_tx);

            let canceled_output_handle_id = format!("{}_canceled_output_{}", node_id, order_config.order_config_id);
            let (canceled_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order canceled output handle: {}", canceled_output_handle_id);
            self.add_output_handle(false, canceled_output_handle_id, canceled_tx);

            let expired_output_handle_id = format!("{}_expired_output_{}", node_id, order_config.order_config_id);
            let (expired_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order expired output handle: {}", expired_output_handle_id);
            self.add_output_handle(false, expired_output_handle_id, expired_tx);

            let rejected_output_handle_id = format!("{}_rejected_output_{}", node_id, order_config.order_config_id);
            let (rejected_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order rejected output handle: {}", rejected_output_handle_id);
            self.add_output_handle(false, rejected_output_handle_id, rejected_tx);

            let error_output_handle_id = format!("{}_error_output_{}", node_id, order_config.order_config_id);
            let (error_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting order error output handle: {}", error_output_handle_id);
            self.add_output_handle(false, error_output_handle_id, error_tx);
        }
    }
}
