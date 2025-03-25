mod utils;
pub mod binance;
use async_trait::async_trait;
use types::market::KlineInterval;
use std::any::Any;



#[async_trait]
pub trait ExchangeClient: Send + Sync + Any {
    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String>;
    async fn get_kline_series(&mut self, symbol: &str, interval: KlineInterval, limit: Option<u32>, start_time: Option<u64>, end_time: Option<u64>) -> Result<(), String>;
    async fn connect_websocket(&mut self) -> Result<(), String>;
    async fn subscribe_kline_stream(&mut self, symbol: &str, interval: KlineInterval) -> Result<(), String>;
    async fn unsubscribe_kline_stream(&mut self, symbol: &str, interval: KlineInterval) -> Result<(), String>;
    async fn get_socket_stream(&mut self) -> Result<(), String>;
    fn as_any(&self) -> &dyn Any;

}