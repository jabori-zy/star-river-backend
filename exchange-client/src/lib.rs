mod utils;
// pub mod binance;
pub mod metatrader5;
pub mod exchange_client_error;
pub mod data_processor_error;

use async_trait::async_trait;
use types::market::KlineInterval;
use types::position::{GetPositionNumberParams, GetPositionParam, PositionNumber, Position,OriginalPosition};
use types::order::{Order, OriginalOrder};
use std::fmt::Debug;
use std::any::Any;
use types::order::{CreateOrderParams, GetTransactionDetailParams};
use types::transaction::{Transaction, OriginalTransaction};
use types::account::OriginalAccountInfo;
use types::market::{Exchange, Kline};
use types::strategy::TimeRange;
use crate::exchange_client_error::ExchangeClientError;

#[async_trait]
pub trait ExchangeClient: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn ExchangeClient>;
    fn exchange_type(&self) -> Exchange;
    async fn connect_websocket(&mut self) -> Result<(), ExchangeClientError>;

    // 市场相关
    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, ExchangeClientError>;
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, ExchangeClientError>;
    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError>;
    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError>;
    async fn get_socket_stream(&self) -> Result<(), ExchangeClientError>;
    async fn get_kline_history(&self, symbol: &str, interval: KlineInterval, time_range: TimeRange) -> Result<Vec<Kline>, ExchangeClientError>;

    //订单相关
    async fn create_order(&self, params: CreateOrderParams) -> Result<Box<dyn OriginalOrder>, ExchangeClientError>; // 发送订单
    async fn update_order(&self, order: Order) -> Result<Order, ExchangeClientError>; // 更新订单

    // 交易明细相关
    async fn get_transaction_detail(&self, params: GetTransactionDetailParams) -> Result<Box<dyn OriginalTransaction>, ExchangeClientError>;

    // 仓位相关
    async fn get_position_number(&self, position_number_request: GetPositionNumberParams) -> Result<PositionNumber, ExchangeClientError>;
    async fn get_position(&self, params: GetPositionParam) -> Result<Box<dyn OriginalPosition>, ExchangeClientError>;
    async fn get_latest_position(&self, position: &Position) -> Result<Position, ExchangeClientError>; // 获取最新持仓

    // 账户相关
    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError>;
}



