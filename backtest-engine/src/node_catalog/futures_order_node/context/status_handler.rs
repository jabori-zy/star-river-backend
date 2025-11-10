
use event_center_core::communication::Response;
use tokio::sync::oneshot;
use snafu::IntoError;

use super::FuturesOrderNodeContext;
use crate::node::node_error::FuturesOrderNodeError;
use star_river_event::communication::GetSymbolInfoCmdPayload;
use star_river_event::communication::GetSymbolInfoCommand;
use star_river_event::communication::MarketEngineCommand;
use strategy_core::node::context_trait::NodeIdentityExt;
use event_center::EventCenterSingleton;
use crate::node::node_error::futures_order_node_error::GetSymbolInfoFailedSnafu;

impl FuturesOrderNodeContext {
    pub(crate) async fn get_symbol_info(&mut self) -> Result<(), FuturesOrderNodeError> {
        let order_config = self.node_config.futures_order_configs.clone();
        let account_id = self.node_config.exchange_mode_config.as_ref().unwrap().selected_account.account_id;
        for order_cfg in order_config {
            let (tx, rx) = oneshot::channel();
            let payload = GetSymbolInfoCmdPayload::new(account_id, order_cfg.symbol.clone());
            let cmd: MarketEngineCommand = GetSymbolInfoCommand::new(
                self.node_id().clone(),
                tx,
                payload
            ).into();
            let _ = EventCenterSingleton::send_command(cmd.into()).await;
            let response = rx.await.unwrap();
            match response {
                Response::Success { payload, datetime } => {
                    if self.symbol_info.contains(&payload.symbol) {
                        continue;
                    }
                    self.symbol_info.push(payload.symbol.clone());
                }
                Response::Fail { error, datetime } => {
                    let e = GetSymbolInfoFailedSnafu {symbol: order_cfg.symbol.clone()}.into_error(error);
                    return Err(e);
                }
            }
        }
        tracing::info!("symbol info: {:#?}", self.symbol_info);
        Ok(())
        
    }
}