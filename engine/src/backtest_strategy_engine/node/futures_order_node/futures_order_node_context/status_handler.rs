use super::{
    FuturesOrderNodeContext,
    FuturesOrderNodeError,
    GetSymbolInfoCmdPayload,
    GetSymbolInfoCommand,
    EventCenterSingleton,
    MarketEngineCommand,
    BacktestNodeContextTrait,
    GetSymbolInfoFailedSnafu
};
use tokio::sync::oneshot;
use event_center::communication::Response;
use snafu::IntoError;


impl FuturesOrderNodeContext {
    pub(crate) async fn get_symbol_info(&mut self) -> Result<(), FuturesOrderNodeError> {
        let order_config = self.node_config.futures_order_configs.clone();
        let account_id = self.node_config.exchange_mode_config.as_ref().unwrap().selected_account.account_id;
        for order_cfg in order_config {
            let (tx, rx) = oneshot::channel();
            let payload = GetSymbolInfoCmdPayload::new(account_id, order_cfg.symbol.clone());
            let cmd: MarketEngineCommand = GetSymbolInfoCommand::new(
                self.get_node_id().clone(),
                tx,
                Some(payload)
            ).into();
            let _ = EventCenterSingleton::send_command(cmd.into()).await;
            let response = rx.await.unwrap();
            if response.is_success() {
                if self.symbol_info.contains(&response.symbol) {
                    continue;
                }
                self.symbol_info.push(response.symbol.clone());
            } else {
                let error = response.get_error();
                let e = GetSymbolInfoFailedSnafu {symbol: order_cfg.symbol.clone()}.into_error(error);
                return Err(e);
            }

        }
        tracing::info!("symbol info: {:#?}", self.symbol_info);
        Ok(())
        
    }
}