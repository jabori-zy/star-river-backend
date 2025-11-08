use super::ExchangeEngineContext;
use star_river_core::account::AccountConfig;
use star_river_core::error::engine_error::*;
use star_river_core::error::exchange_client_error::*;
use star_river_core::market::{Exchange, ExchangeStatus};
use exchange_client::metatrader5::{MetaTrader5, Mt5Metadata};
use snafu::ResultExt;


impl ExchangeEngineContext {
    pub(super) async fn register_mt5_exchange_in_dev(&mut self, account_config: AccountConfig) -> Result<(), ExchangeEngineError> {
        let login = account_config.config["login"].as_i64().unwrap();
        let password = account_config.config["password"].as_str().unwrap().to_string();
        let server = account_config.config["server"].as_str().unwrap().to_string();
        let terminal_path = account_config.config["terminal_path"].as_str().unwrap().to_string();

        let mut mt5 = MetaTrader5::new(
            account_config.id,
            login,
            password,
            server.clone(),
            terminal_path,
        );

        match mt5.connect_to_server(8001).await {
            Ok(_) => tracing::info!("mt5 server connect success, port: 8001"),
            Err(e) => {
                tracing::error!("context1: {}", e);
                let exchange_client_error = ExchangeClientError::from(e);
                return Err(exchange_client_error).context(RegisterExchangeFailedSnafu {
                    message: "fail to connect to server".to_string(),
                    account_id: account_config.id,
                    exchange_type: Exchange::Metatrader5(server.clone()),
                })?;
            }
        }

        match mt5.initialize_terminal().await {
            Ok(_) => {
                tracing::info!(account_id = %account_config.id, "mt5 terminal is initialized successfully")
            }
            Err(e) => {
                tracing::error!("context2: {}", e);
                let exchange_client_error = ExchangeClientError::from(e);
                return Err(exchange_client_error).context(RegisterExchangeFailedSnafu {
                    message: "fail to initialize terminal".to_string(),
                    account_id: account_config.id,
                    exchange_type: Exchange::Metatrader5(server.clone()),
                })?;
            }
        }

        match mt5.connect_websocket().await {
            Ok(_) => tracing::info!("MT5-{} websocket connect success", account_config.id),
            Err(e) => {
                tracing::error!("context3: {}", e);
                let exchange_client_error = ExchangeClientError::from(e);
                return Err(exchange_client_error).context(RegisterExchangeFailedSnafu {
                    message: "fail to connect to websocket".to_string(),
                    account_id: account_config.id,
                    exchange_type: Exchange::Metatrader5(server.clone()),
                })?;
            }
        }

        mt5.set_status(ExchangeStatus::Connected);
        let mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClientCore>;

        tracing::info!("MT5-{} exchange register success!", account_config.id);
        self.exchanges.insert(account_config.id, mt5_exchange);
        Ok(())
    }
}
