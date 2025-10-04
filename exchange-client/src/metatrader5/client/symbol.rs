use super::{
    ExchangeSymbolExt,
    MetaTrader5,
    KlineInterval,
    Mt5KlineInterval,
    ExchangeClientError,
    HttpClientNotCreatedSnafu,
};
use async_trait::async_trait;
use star_river_core::market::Symbol;


#[async_trait]
impl ExchangeSymbolExt for MetaTrader5 {
    async fn get_symbol_list(&self) -> Result<Vec<Symbol>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let symbols = mt5_http_client.get_symbol_list().await?;
            let data_processor = self.data_processor.lock().await;
            let symbols = data_processor.process_symbol_list(symbols).await?;
            Ok(symbols)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_symbol(&self, symbol: String) -> Result<Symbol, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let symbol = mt5_http_client.get_symbol_info(&symbol).await?;
            let data_processor = self.data_processor.lock().await;
            let symbol = data_processor.process_symbol(symbol).await?;
            Ok(symbol)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    fn get_support_kline_intervals(&self) -> Vec<KlineInterval> {
        Mt5KlineInterval::to_list()
            .iter()
            .map(|interval| KlineInterval::from(interval.clone()))
            .collect()
    }
}