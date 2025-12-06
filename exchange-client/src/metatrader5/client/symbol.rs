use async_trait::async_trait;
use exchange_core::exchange_trait::{ExchangeSymbolExt, ProcessorAccessor};
use star_river_core::{instrument::Symbol, kline::KlineInterval};

use super::error::Mt5Error;
use crate::metatrader5::{MetaTrader5, mt5_types::Mt5KlineInterval};

#[async_trait]
impl ExchangeSymbolExt for MetaTrader5 {
    type Error = Mt5Error;

    async fn symbol_list(&self) -> Result<Vec<Symbol>, Mt5Error> {
        let symbols_info = self.http_client().get_symbol_list().await?;

        // Use processor accessor to process symbol list
        let symbols = self
            .with_processor_read_async(|processor| Box::pin(async move { processor.process_symbol_list(symbols_info) }))
            .await?;
        Ok(symbols)
    }

    async fn symbol(&self, symbol: String) -> Result<Symbol, Mt5Error> {
        let symbol_info = self.http_client().get_symbol_info(&symbol).await?;

        // Use processor accessor to process symbol
        let symbol = self
            .with_processor_read_async(|processor| Box::pin(async move { processor.process_symbol(symbol_info) }))
            .await?;
        tracing::debug!("symbol: {:?}", symbol);
        Ok(symbol)
    }

    fn support_kline_intervals(&self) -> Vec<KlineInterval> {
        Mt5KlineInterval::to_list()
            .iter()
            .map(|interval| KlineInterval::from(interval.clone()))
            .collect()
    }
}
