use async_trait::async_trait;
use exchange_core::exchange_trait::{ExchangeMarketDataExt, ProcessorAccessor};
use star_river_core::{
    kline::{Kline, KlineInterval},
    system::TimeRange,
};

use super::error::Mt5Error;
use crate::metatrader5::{MetaTrader5, mt5_types::Mt5KlineInterval};

#[async_trait]
impl ExchangeMarketDataExt for MetaTrader5 {
    type Error = Mt5Error;

    async fn kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, Self::Error> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.http_client();

        let kline_series = mt5_http_client.get_kline_series(symbol, mt5_interval.clone(), limit).await?;

        let symbol_owned = symbol.to_string();
        let kline_series_result = self
            .with_processor_read_async(|p| {
                Box::pin(async move {
                    // Move ownership of mt5_interval and kline_series into the async block
                    // Use reference to the owned symbol
                    p.process_kline_series(&symbol_owned, mt5_interval, kline_series).await
                })
            })
            .await?;
        Ok(kline_series_result)
    }

    async fn kline_history(&self, symbol: &str, interval: KlineInterval, time_range: TimeRange) -> Result<Vec<Kline>, Self::Error> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.http_client();

        let kline_history = mt5_http_client.get_kline_history(symbol, mt5_interval.clone(), time_range).await?;

        let symbol_owned = symbol.to_string();
        let klines_history_result = self
            .with_processor_read_async(|p| {
                Box::pin(async move { p.process_kline_series(&symbol_owned, mt5_interval, kline_history).await })
            })
            .await?;
        Ok(klines_history_result)
    }
}
