
use star_river_core::market::Symbol;
use async_trait::async_trait;
use exchange_core::exchange_trait::{ExchangeSymbolExt, ProcessorAccessor};
use star_river_core::market::KlineInterval;
use crate::binance::{Binance, error::BinanceError};
use crate::binance::binance_type::BinanceKlineInterval;

#[async_trait]
impl ExchangeSymbolExt for Binance {
    type Error = BinanceError;
    async fn symbol_list(&self) -> Result<Vec<Symbol>, Self::Error> {
        let exchange_info = self.http_client().get_exchange_info().await?;

        // Use processor accessor to process symbol list
        let symbols = self.with_processor_read_async(|processor|
            Box::pin(async move {
                processor.process_symbol_list(exchange_info)
            })
        ).await?;
        Ok(symbols)

    }

    async fn symbol(&self, symbol: String) -> Result<Symbol, Self::Error> {
        let symbol_info = self.http_client().get_symbol_info(&symbol).await?;

        // Use processor accessor to process symbol
        let symbol = self.with_processor_read_async(|processor|
            Box::pin(async move {
                processor.process_symbol(symbol_info)
            })
        ).await?;
        tracing::debug!("symbol: {:?}", symbol);
        Ok(symbol)
    }

    fn support_kline_intervals(&self) -> Vec<KlineInterval> {
        BinanceKlineInterval::to_list()
            .iter()
            .map(|interval| KlineInterval::from(interval.clone()))
            .collect()
    }
}
