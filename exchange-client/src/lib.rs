mod utils;
pub mod binance;
pub mod metatrader5;

use async_trait::async_trait;
use types::market::KlineInterval;
use std::any::Any;
use types::order::{OrderType, OrderSide, OrderRequest, Order};
use std::collections::HashMap;
use types::market::Exchange;


pub trait ExchangeClient: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
}


#[async_trait]
pub trait Market: Send + Sync + Any {
    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String>;
    async fn get_kline_series(&mut self, symbol: &str, interval: KlineInterval, limit: Option<u32>) -> Result<(), String>;
    async fn connect_websocket(&mut self) -> Result<(), String>;
    async fn subscribe_kline_stream(&mut self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn unsubscribe_kline_stream(&mut self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn get_socket_stream(&mut self) -> Result<(), String>;
    fn as_any(&self) -> &dyn Any;

}


#[async_trait]
pub trait Trading: Send + Sync + Any {
    async fn open_long(&mut self, order_type: OrderType, symbol: &str, quantity: f64, price: f64, tp: Option<f64>, sl: Option<f64>) -> Result<Order, String>; // 开多仓
    // async fn open_short(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String>; // 开空仓
    // async fn close_long(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String>; // 平多仓
    // async fn close_short(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String>; // 平空仓
    fn as_any(&self) -> &dyn Any;
}



pub struct ExchangeManager {
    pub exchanges: HashMap<Exchange, Box<dyn ExchangeClient>>,
}


