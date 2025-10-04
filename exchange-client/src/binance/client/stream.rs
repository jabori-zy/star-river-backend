use super::{
    BinanceExchange,
    ExchangeStreamExt,
    KlineInterval,
    BinanceKlineInterval,
    ExchangeClientError,
    async_trait,
};
use crate::binance::market_stream::klines;
use futures::StreamExt;
use std::sync::atomic::Ordering;

#[async_trait]
impl ExchangeStreamExt for BinanceExchange {
    async fn connect_websocket(&mut self) -> Result<(), ExchangeClientError> {
        use crate::binance::binance_ws_client::BinanceWsClient;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let (websocket_state, _) = BinanceWsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        Ok(())
    }

    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError> {
        let _frequency = frequency;
        let binance_interval = BinanceKlineInterval::from(interval.clone());

        let mut websocket_state = self.websocket_state.lock().await;
        if let Some(state) = websocket_state.as_mut() {
            tracing::debug!("Subscribe kline stream, symbol: {:?}, interval: {:?}", symbol, interval);
            state.subscribe([&klines(symbol, binance_interval).into()]).await;
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError> {
        let _frequency = frequency;
        let binance_interval = BinanceKlineInterval::from(interval.clone());
        let mut websocket_state = self.websocket_state.lock().await;
        if let Some(state) = websocket_state.as_mut() {
            tracing::debug!("Unsubscribe kline stream, symbol: {:?}, interval: {:?}", symbol, interval);
            state.unsubscribe([&klines(symbol, binance_interval).into()]).await;
        }
        Ok(())
    }

    async fn get_socket_stream(&self) -> Result<(), ExchangeClientError> {
        // Check if stream processing is already running
        if self.is_process_stream.load(Ordering::Relaxed) {
            tracing::warn!("Binance stream processing already started!");
            return Ok(());
        }
        tracing::debug!("Starting binance stream processing");
        // If not processing stream, start it and set status to true
        self.is_process_stream.store(true, Ordering::Relaxed);

        let websocket_state = self.websocket_state.clone();
        let data_processor = self.data_processor.clone();

        // let binance_publisher = self.event_publisher.clone();
        let future = async move {
            loop {
                let receive_message = {
                    let mut websocket_state = websocket_state.lock().await;
                    if let Some(state) = websocket_state.as_mut() {
                        state.as_mut().next().await
                    } else {
                        None
                    }
                }; // Lock is released here

                // Process raw data
                if let Some(Ok(msg)) = receive_message {
                    let stream_json = serde_json::from_str::<serde_json::Value>(&msg.to_string()).unwrap();
                    // log::debug!("Received stream data: {:?}", stream_json);
                    let data_processor = data_processor.lock().await;
                    // data_processor.process_stream(stream_json, binance_publisher.clone()).await;
                }
            }
        };
        tokio::spawn(future);
        Ok(())
    }
}
