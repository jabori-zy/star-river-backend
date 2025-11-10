use async_trait::async_trait;
use exchange_core::exchange_trait::{ExchangeMarketDataExt, ProcessorAccessor};
use star_river_core::{
    kline::{Kline, KlineInterval},
    system::TimeRange,
};

use super::error::BinanceError;
use crate::binance::{Binance, binance_type::BinanceKlineInterval};

#[async_trait]
impl ExchangeMarketDataExt for Binance {
    type Error = BinanceError;

    async fn kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, Self::Error> {
        let binance_interval = BinanceKlineInterval::from(interval);
        let binance_http_client = self.http_client();

        let klines = binance_http_client
            .get_kline(symbol, binance_interval.clone(), Some(limit), None, None)
            .await?;

        // Use processor accessor to process kline data
        let klines_result = self
            .with_processor_read_async(|processor| Box::pin(async move { processor.process_kline_series(klines).await }))
            .await?;
        Ok(klines_result)
    }

    async fn kline_history(&self, symbol: &str, interval: KlineInterval, time_range: TimeRange) -> Result<Vec<Kline>, Self::Error> {
        let binance_interval = BinanceKlineInterval::from(interval);
        let binance_http_client = self.http_client();

        let klines = binance_http_client
            .get_kline(
                symbol,
                binance_interval,
                None,
                Some(time_range.start_date.timestamp_millis() as u64),
                Some(time_range.end_date.timestamp_millis() as u64),
            )
            .await?;

        // Use processor accessor to process kline data
        let klines_result = self
            .with_processor_read_async(|processor| Box::pin(async move { processor.process_kline_series(klines).await }))
            .await?;
        Ok(klines_result)
    }
}
