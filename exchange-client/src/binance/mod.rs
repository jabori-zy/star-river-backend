pub mod binance_data_processor;
pub mod binance_http_client;
pub mod binance_ws_client;
pub mod market_stream;
pub(crate) mod url;
pub mod websocket;
mod binance_type;
mod client;

#[cfg(test)]
mod test;
use std::any::Any;
use std::sync::atomic::AtomicBool;

use crate::binance::binance_data_processor::BinanceDataProcessor;
use async_trait::async_trait;
use binance_http_client::BinanceHttpClient;
use binance_ws_client::WebSocketState;
use event_center::EventPublisher;
use star_river_core::market::{Exchange, Kline, KlineInterval};
use std::sync::Arc;
use tokio::sync::Mutex;
use binance_type::*;
use super::exchange_trait::*;
use star_river_core::error::exchange_client_error::binance_error::*;





// 交易所
#[derive(Clone, Debug)]
pub struct BinanceExchange {
    http_client: BinanceHttpClient,
    websocket_state: Arc<Mutex<Option<WebSocketState>>>, // 可以在线程间传递
    data_processor: Arc<Mutex<BinanceDataProcessor>>,
    is_process_stream: Arc<AtomicBool>
}

#[async_trait]
impl ExchangeClientCore for BinanceExchange {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ExchangeClientCore> {
        Box::new(self.clone())
    }

    fn exchange_type(&self) -> Exchange {
        Exchange::Binance
    }

}

impl BinanceExchange {
    pub fn new() -> Self {
        Self {
            http_client: BinanceHttpClient::new(),
            websocket_state: Arc::new(Mutex::new(None)),
            data_processor: Arc::new(Mutex::new(BinanceDataProcessor{})),
            is_process_stream: Arc::new(AtomicBool::new(false))
        }
    }

    pub async fn init_exchange(&mut self) -> Result<(), String> {
        use crate::binance::binance_ws_client::BinanceWsClient;

        tracing::debug!("Initializing Binance exchange...");
        let (websocket_state, _) = BinanceWsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        tracing::debug!("Binance initialized successfully!");
        Ok(())
    }
}
