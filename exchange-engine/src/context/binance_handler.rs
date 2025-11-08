use super::{ExchangeEngineContext};
use star_river_core::account::AccountConfig;
use star_river_core::exchange::Exchange;
use exchange_client_new::binance::Binance;
use exchange_client_new::binance::BinanceMetadata;
use exchange_core::exchange_trait::ExchangeLifecycle;
use crate::error::{ExchangeEngineError, RegisterExchangeFailedSnafu};
use snafu::ResultExt;
use exchange_client_new::exchange_error::ExchangeError;

impl ExchangeEngineContext {
    pub(super) async fn register_binance_exchange(&mut self, account_config: AccountConfig) -> Result<(), ExchangeEngineError> {
        let metadata = BinanceMetadata::new(account_config.id, account_config.account_name);
        let binance = Binance::new(metadata);

        // Initialize binance and convert error chain: BinanceError -> ExchangeClientError -> ExchangeEngineError
        binance.initialize().await.map_err(|e| {
            ExchangeError::from(e)
        }).context(RegisterExchangeFailedSnafu {
            account_id: account_config.id,
            exchange_type: Exchange::Binance,
        })?;

        self.exchanges.insert(account_config.id, binance.into());
        Ok(())
    }
}
