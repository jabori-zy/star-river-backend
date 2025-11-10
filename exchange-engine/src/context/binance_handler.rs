use exchange_client::{
    binance::{Binance, BinanceMetadata},
    exchange_error::ExchangeError,
};
use exchange_core::exchange_trait::ExchangeLifecycle;
use snafu::ResultExt;
use star_river_core::{account::AccountConfig, exchange::Exchange};

use super::ExchangeEngineContext;
use crate::error::{ExchangeEngineError, RegisterExchangeFailedSnafu};

impl ExchangeEngineContext {
    pub(super) async fn register_binance_exchange(&mut self, account_config: AccountConfig) -> Result<(), ExchangeEngineError> {
        let metadata = BinanceMetadata::new(account_config.id, account_config.account_name);
        let binance = Binance::new(metadata);

        // Initialize binance and convert error chain: BinanceError -> ExchangeClientError -> ExchangeEngineError
        binance
            .initialize()
            .await
            .map_err(|e| ExchangeError::from(e))
            .context(RegisterExchangeFailedSnafu {
                account_id: account_config.id,
                exchange_type: Exchange::Binance,
            })?;

        self.exchanges.insert(account_config.id, binance.into());
        Ok(())
    }
}
