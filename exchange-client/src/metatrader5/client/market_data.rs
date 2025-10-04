use super::{
    ExchangeMarketDataExt,
    MetaTrader5,
    KlineInterval,
    Kline,
    ExchangeClientError,
    Mt5KlineInterval,
    HttpClientNotCreatedSnafu,
    async_trait,
    TimeRange,
};


#[async_trait]
impl ExchangeMarketDataExt for MetaTrader5 {
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let kline_series = mt5_http_client.get_kline_series(symbol, mt5_interval.clone(), limit).await?;
            let data_processor = self.data_processor.lock().await;
            let kline_series = data_processor.process_kline_series(symbol, mt5_interval, kline_series).await?;
            Ok(kline_series)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_kline_history(
        &self,
        symbol: &str,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let kline_history = mt5_http_client.get_kline_history(symbol, mt5_interval.clone(), time_range).await?;
            let data_processor = self.data_processor.lock().await;
            let klines = data_processor.process_kline_series(symbol, mt5_interval, kline_history).await?;
            Ok(klines)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

}