mod utils;
pub mod binance;
pub mod metatrader5;

use async_trait::async_trait;
use types::market::KlineInterval;
use types::position::{PositionNumberRequest, PositionNumber};
use types::order::{OrderType, OrderSide, OrderRequest, Order};
use std::collections::HashMap;
use types::market::Exchange;
use tokio::sync::RwLock;
use tokio::sync::{RwLockReadGuard,RwLockWriteGuard};
use std::sync::Arc;
use event_center::EventPublisher;
use crate::binance::BinanceExchange;
use crate::metatrader5::MetaTrader5;
use event_center::Event;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use event_center::request_event::CommandEvent;
use event_center::request_event::ExchangeManagerCommand;
use event_center::request_event::RegisterExchangeParams;
use event_center::response_event::ResponseEvent;
use event_center::response_event::ExchangeManagerResponse;
use event_center::response_event::RegisterExchangeSuccessResponse;
use utils::get_utc8_timestamp;
use std::fmt::Debug;
use std::any::Any;

#[async_trait]
pub trait ExchangeClient: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn ExchangeClient>;
    async fn connect_websocket(&mut self) -> Result<(), String>;

    // 市场相关
    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String>;
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: Option<u32>) -> Result<(), String>;
    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn get_socket_stream(&self) -> Result<(), String>;

    //创建订单
    async fn create_order(&self, order_request: OrderRequest) -> Result<Order, String>; // 发送订单
    // 获取仓位个数
    async fn get_position_number(&self, position_number_request: PositionNumberRequest) -> Result<PositionNumber, String>;

}



