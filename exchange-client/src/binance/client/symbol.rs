use super::{
    ExchangeSymbolExt,
    BinanceExchange,
    KlineInterval,
    BinanceKlineInterval,
    ExchangeClientError,
    async_trait,
};
use star_river_core::market::Symbol;

#[async_trait]
impl ExchangeSymbolExt for BinanceExchange {
    async fn get_symbol_list(&self) -> Result<Vec<Symbol>, ExchangeClientError> {
        todo!()
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
