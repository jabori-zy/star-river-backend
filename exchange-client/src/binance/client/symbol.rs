use super::{
    ExchangeSymbolExt,
    Binance,
    KlineInterval,
    BinanceKlineInterval,
    ExchangeClientError,
    async_trait,
};
use star_river_core::market::Symbol;

#[async_trait]
impl ExchangeSymbolExt for Binance {
    async fn get_symbol_list(&self) -> Result<Vec<Symbol>, ExchangeClientError> {
        let exchange_info = self.http_client.get_exchange_info().await?;
        let processor = self.data_processor.lock().await;
        let symbols = processor.process_symbol_list(exchange_info)?;
        Ok(symbols)

    }

    async fn get_symbol(&self, _symbol: String) -> Result<Symbol, ExchangeClientError> {
        todo!()
    }

    fn get_support_kline_intervals(&self) -> Vec<KlineInterval> {
        BinanceKlineInterval::to_list()
            .iter()
            .map(|interval| KlineInterval::from(interval.clone()))
            .collect()
    }
}
