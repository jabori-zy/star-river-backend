use database::query::account_config_query::AccountConfigQuery;
use snafu::Report;
use star_river_core::{custom_type::AccountId, exchange::Exchange};

use super::ExchangeEngineContext;
use crate::error::{ExchangeEngineError, UnsupportedExchangeTypeSnafu};

impl ExchangeEngineContext {
    pub async fn register_exchange(&mut self, account_id: AccountId) -> Result<(), ExchangeEngineError> {
        tracing::debug!("start to register exchange, account id is {}", account_id);

        if self.exchanges.contains_key(&account_id) {
            tracing::debug!("account {} already registered", account_id);
            return Ok(());
        }

        let account_config = AccountConfigQuery::get_account_config_by_id(&self.database, account_id).await?;

        let result = match account_config.exchange {
            // Exchange::Metatrader5(_) => {
            //     #[cfg(not(debug_assertions))]
            //     {
            //         Self::register_mt5_exchange(self, account_config).await
            //     }
            //     #[cfg(debug_assertions)]
            //     {
            //         tracing::debug!("in the dev mode, direct connect to mt5 server");
            //         self.register_mt5_exchange_in_dev(account_config).await
            //     }
            // }
            Exchange::Binance => self.register_binance_exchange(account_config).await,
            _ => {
                let error = UnsupportedExchangeTypeSnafu {
                    exchange_type: account_config.exchange.clone(),
                    account_id,
                }
                .build();
                tracing::error!("{}", error);
                return Err(error);
            }
        };

        match result {
            Ok(()) => {
                tracing::info!("account {account_id}'s exchange register success");
                Ok(())
            }
            Err(e) => {
                let report = Report::from_error(&e);
                tracing::error!("{}", report);
                Err(e)
            }
        }
    }

    pub async fn unregister_exchange(&mut self, account_id: AccountId) -> Result<(), ExchangeEngineError> {
        self.exchanges.remove(&account_id);

        // let mut exchange = self.get_exchange_instance(&account_id).await?;
        // match exchange.exchange_type() {
        // Exchange::Metatrader5(_) => {
        //     let mt5 = exchange.as_any_mut().downcast_mut::<MetaTrader5>().unwrap();

        //     match tokio::time::timeout(tokio::time::Duration::from_secs(15), mt5.stop_mt5_server()).await {
        //         Ok(result) => match result {
        //             Ok(true) => {
        //                 tracing::info!("MT5 server stopped successfully, account_id: {}", account_id);
        //                 self.exchanges.remove(&account_id);
        //             }
        //             Ok(false) => {
        //                 tracing::error!("MT5 server stop failed, but still remove instance, account_id: {}", account_id);
        //                 self.exchanges.remove(&account_id);
        //             }
        //             Err(e) => {
        //                 tracing::error!("MT5 server stop error: {}, account_id: {}", e, account_id);
        //                 self.exchanges.remove(&account_id);
        //             }
        //         },
        //         Err(_) => {
        //             tracing::error!("MT5 server stop timeout, account_id: {}", account_id);
        //             self.exchanges.remove(&account_id);
        //         }
        //     }
        // }
        // _ => {
        //     self.exchanges.remove(&account_id);
        // }

        Ok(())
    }
}
