use exchange_client::binance::{Binance, BinanceMetadata};
use exchange_core::exchange_trait::ExchangeLifecycle;
use snafu::ResultExt;
use star_river_core::account::AccountConfig;

use super::ExchangeEngineContext;
use crate::error::{BinanceRegisterFailedSnafu, ExchangeEngineError};

impl ExchangeEngineContext {
    pub(super) async fn register_binance_exchange(&mut self, account_config: AccountConfig) -> Result<(), ExchangeEngineError> {
        let metadata = BinanceMetadata::new(account_config.id, account_config.account_name.clone());
        let binance = Binance::new(metadata);

        // Initialize binance and convert error chain: BinanceError -> ExchangeClientError -> ExchangeEngineError
        binance.initialize().await.context(BinanceRegisterFailedSnafu {
            exchange_name: account_config.account_name,
        })?;

        self.exchanges.insert(account_config.id, binance.into());
        Ok(())
    }
}
