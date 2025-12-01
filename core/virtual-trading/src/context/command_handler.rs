use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use super::VirtualTradingSystemContext;
use crate::command::{
    CloseAllPositionsRespPayload, CloseAllPositionsResponse, ClosePositionRespPayload, ClosePositionResponse, CreateOrderRespPayload,
    CreateOrderResponse, VtsCommand,
};

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn get_command_receiver(&self) -> Arc<Mutex<mpsc::Receiver<VtsCommand>>> {
        self.command_transceiver.1.clone()
    }

    pub fn get_command_sender(&self) -> &mpsc::Sender<VtsCommand> {
        &self.command_transceiver.0
    }

    pub async fn handle_command(&mut self, command: VtsCommand) {
        match command {
            VtsCommand::CreateOrder(cmd) => {
                let result = self.create_order(
                    cmd.strategy_id,
                    cmd.node_id.clone(),
                    cmd.node_name.clone(),
                    cmd.order_config_id,
                    cmd.symbol.clone(),
                    cmd.exchange.clone(),
                    cmd.price,
                    cmd.order_side.clone(),
                    cmd.order_type.clone(),
                    cmd.quantity,
                    cmd.tp,
                    cmd.sl,
                    cmd.tp_type.clone(),
                    cmd.sl_type.clone(),
                    cmd.point,
                );
                match result {
                    Ok(()) => {
                        let payload = CreateOrderRespPayload::new(1);
                        let response = CreateOrderResponse::success(payload);
                        cmd.respond(response);
                    }
                    Err(e) => {
                        let response = CreateOrderResponse::fail(e);
                        cmd.respond(response);
                    }
                }
            }
            VtsCommand::ClosePosition(cmd) => {
                let result = self.close_position(&cmd.node_id, &cmd.node_name, cmd.config_id, &cmd.symbol, &cmd.exchange);
                match result {
                    Ok(position_id) => {
                        let payload = ClosePositionRespPayload::new(position_id);
                        let response = ClosePositionResponse::success(payload);
                        cmd.respond(response);
                    }
                    Err(e) => {
                        let response = ClosePositionResponse::fail(e);
                        cmd.respond(response);
                    }
                }
            }
            VtsCommand::CloseAllPositions(cmd) => {
                let result = self.close_all_positions(&cmd.node_id, &cmd.node_name, cmd.config_id);
                match result {
                    Ok(position_ids) => {
                        let payload = CloseAllPositionsRespPayload::new(position_ids);
                        let response = CloseAllPositionsResponse::success(payload);
                        cmd.respond(response);
                    }
                    Err(e) => {
                        let response = CloseAllPositionsResponse::fail(e);
                        cmd.respond(response);
                    }
                }
            }
        }
    }
}
