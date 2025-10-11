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
use star_river_core::market::{Exchange, ExchangeStatus, Kline, KlineInterval};
use std::sync::Arc;
use tokio::sync::Mutex;
use binance_type::*;
use super::exchange_trait::*;
use star_river_core::error::exchange_client_error::binance_error::*;
use crate::binance::binance_ws_client::BinanceWsClient;




// 交易所
#[derive(Clone, Debug)]
pub struct Binance {
    http_client: BinanceHttpClient,
    websocket_state: Arc<Mutex<Option<WebSocketState>>>, // 可以在线程间传递
    data_processor: Arc<Mutex<BinanceDataProcessor>>,
    is_process_stream: Arc<AtomicBool>,
    status: ExchangeStatus,
}

#[async_trait]
impl ExchangeClientCore for Binance {
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

    fn get_status(&self) -> ExchangeStatus {
        self.status.clone()
    }

    fn set_status(&mut self, status: ExchangeStatus) {
        self.status = status;
    }

}

impl Binance {
    pub fn new() -> Self {
        Self {
            http_client: BinanceHttpClient::new(),
            websocket_state: Arc::new(Mutex::new(None)),
            data_processor: Arc::new(Mutex::new(BinanceDataProcessor{})),
            is_process_stream: Arc::new(AtomicBool::new(false)),
            status: ExchangeStatus::NotRegist,
        }
    }

    pub async fn init_exchange(&mut self) -> Result<(), String> {
        self.status = ExchangeStatus::Registing;
        let (websocket_state, _) = BinanceWsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        self.status = ExchangeStatus::Connected;
        tracing::debug!("Binance initialized successfully!");
        Ok(())
    }
}
