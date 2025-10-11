use async_trait::async_trait;
use star_river_core::account::OriginalAccountInfo;
use star_river_core::error::exchange_client_error::ExchangeClientError;
use star_river_core::market::KlineInterval;
use star_river_core::market::Symbol;
use star_river_core::market::{Exchange, Kline};
use star_river_core::order::{CreateOrderParams, GetTransactionDetailParams};
use star_river_core::order::{Order, OriginalOrder};
use star_river_core::position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position, PositionNumber};
use star_river_core::strategy::TimeRange;
use star_river_core::transaction::OriginalTransaction;
use std::any::Any;
use std::fmt::Debug;
use star_river_core::market::ExchangeStatus;

/// Core exchange client trait
/// Provides basic type conversion and exchange identification
pub trait ExchangeClientCore:
    ExchangeMarketDataExt
    + ExchangeStreamExt
    + ExchangeSymbolExt
    + ExchangeOrderExt
    + ExchangePositionExt
    + ExchangeAccountExt
    + Debug
    + Send
    + Sync
    + Any
    + 'static
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn ExchangeClientCore>;
    fn exchange_type(&self) -> Exchange;
    fn get_status(&self) -> ExchangeStatus;
    fn set_status(&mut self, status: ExchangeStatus);
    
}

/// Market data query trait
/// Responsible for fetching real-time and historical market data
#[async_trait]
pub trait ExchangeMarketDataExt {
    /// Get the latest ticker price for a symbol
    // async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, ExchangeClientError>;

    /// Get kline series data
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32) -> Result<Vec<Kline>, ExchangeClientError>;

    /// Get historical kline data
    async fn get_kline_history(
        &self,
        symbol: &str,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, ExchangeClientError>;
}

/// WebSocket stream subscription trait
/// Responsible for real-time data stream subscription and management
#[async_trait]
pub trait ExchangeStreamExt {
    /// Connect to WebSocket
    async fn connect_websocket(&mut self) -> Result<(), ExchangeClientError>;

    /// Subscribe to kline data stream
    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError>;

    /// Unsubscribe from kline data stream
    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError>;

    /// Get socket stream data
    async fn get_socket_stream(&self) -> Result<(), ExchangeClientError>;
}

/// Symbol management trait
/// Responsible for querying and managing trading pair information
#[async_trait]
pub trait ExchangeSymbolExt {
    /// Get list of all supported trading symbols
    async fn get_symbol_list(&self) -> Result<Vec<Symbol>, ExchangeClientError>;

    /// Get information for a specific trading symbol
    async fn get_symbol(&self, symbol: String) -> Result<Symbol, ExchangeClientError>;

    /// Get supported kline intervals
    fn get_support_kline_intervals(&self) -> Vec<KlineInterval>;
}

/// Order management trait
/// Responsible for creating and updating orders
#[async_trait]
pub trait ExchangeOrderExt {
    /// Create an order
    async fn create_order(&self, params: CreateOrderParams) -> Result<Box<dyn OriginalOrder>, ExchangeClientError>;

    /// Update an order
    async fn update_order(&self, order: Order) -> Result<Order, ExchangeClientError>;
    /// Get transaction detail
    async fn get_transaction_detail(&self, params: GetTransactionDetailParams)
    -> Result<Box<dyn OriginalTransaction>, ExchangeClientError>;
}

/// Position management trait
/// Responsible for querying and managing position information
#[async_trait]
pub trait ExchangePositionExt {
    /// Get position number information
    async fn get_position_number(&self, params: GetPositionNumberParams) -> Result<PositionNumber, ExchangeClientError>;

    /// Get position details
    async fn get_position(&self, params: GetPositionParam) -> Result<Box<dyn OriginalPosition>, ExchangeClientError>;

    /// Get latest position information
    async fn get_latest_position(&self, position: &Position) -> Result<Position, ExchangeClientError>;
}

/// Account management trait
/// Responsible for querying account information and transaction details
#[async_trait]
pub trait ExchangeAccountExt {
    /// Get account information
    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError>;
}
