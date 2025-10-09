use super:: {
    ExchangeMarketDataExt,
    Binance,
    KlineInterval,
    Kline,
    ExchangeClientError,
    TimeRange,
    BinanceKlineInterval,
    async_trait,
};

#[async_trait]
impl ExchangeMarketDataExt for Binance {
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, ExchangeClientError> {
        let binance_interval = BinanceKlineInterval::from(interval);

        let klines = self
            .http_client
            .get_kline(symbol, binance_interval.clone(), Some(limit), None, None)
            .await?;
        // 发送到数据处理器，处理数据
        let data_processor = self.data_processor.lock().await;
        let klines = data_processor
            .process_kline_series(klines)
            .await?;
        Ok(klines)
    }

    async fn get_kline_history(
        &self,
        symbol: &str,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, ExchangeClientError> {
        let binance_interval = BinanceKlineInterval::from(interval);
        let klines = self.http_client
            .get_kline(symbol, binance_interval, None, Some(time_range.start_date.timestamp_millis() as u64), Some(time_range.end_date.timestamp_millis() as u64))
        .await?;
        let data_processor = self.data_processor.lock().await;
        let klines = data_processor.process_kline_series(klines).await?;
        Ok(klines)
    }
}