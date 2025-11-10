use async_trait::async_trait;
use star_river_core::{
    account::OriginalAccountInfo,
    error::StarRiverErrorTrait,
    exchange::Exchange as ExchangeType,
    kline::{Kline, KlineInterval},
    position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position, PositionNumber},
    instrument::Symbol,
};

use star_river_core::system::TimeRange;


use crate::{ExchangeRunState, state_machine::ExchangeStateTransTrigger};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Base Component Trait Definitions
// ============================================================================

/// HTTP client interface
///
/// Defines the common interface for exchange HTTP clients
pub trait HttpClient: Debug + Send + Sync + 'static {}

/// WebSocket client interface
///
/// Defines the common interface for exchange WebSocket clients
pub trait WebSocketClient: Debug + Send + Sync + 'static {}

/// Data processor interface
///
/// Defines the common interface for exchange data processors
pub trait DataProcessor: Debug + Send + Sync + 'static {}

/// Exchange metadata
///
/// Each exchange can define its own metadata structure
/// Examples:
/// - Binance: is_process_stream
/// - MT5: server, account, terminal_id
pub trait ExchangeMetadata: Debug + Send + Sync + 'static {}

// ============================================================================
// Component Accessor Traits
// ============================================================================

/// Metadata accessor trait
///
/// Provides convenient methods to access exchange metadata with proper lock management
///
/// # Associated Types
/// - `Metadata`: Exchange-specific metadata type, must implement `ExchangeMetadata`
///
/// # Example
///
/// ```rust,ignore
/// impl MetadataAccessor for Binance {
///     type Metadata = BinanceMetadata;
///
///     fn metadata(&self) -> &Arc<RwLock<Self::Metadata>> {
///         &self.metadata
///     }
/// }
///
/// // Read metadata (sync closure)
/// let is_processing = exchange.with_metadata_read(|meta| {
///     meta.is_process_stream()
/// }).await;
///
/// // Write metadata (sync closure)
/// exchange.with_metadata_write(|meta| {
///     meta.set_process_stream(true);
/// }).await;
///
/// // Read metadata (async closure)
/// exchange.with_metadata_read_async(|meta| {
///     Box::pin(async move {
///         meta.fetch_config().await
///     })
/// }).await;
///
/// // Write metadata (async closure)
/// exchange.with_metadata_write_async(|meta| {
///     Box::pin(async move {
///         meta.update_config().await;
///     })
/// }).await;
/// ```
#[async_trait]
pub trait MetadataAccessor: Send + Sync {
    /// Exchange-specific metadata type
    type Metadata: ExchangeMetadata;

    /// Get metadata reference
    ///
    /// # Returns
    /// Returns an `Arc<RwLock<Metadata>>` reference to the metadata
    fn metadata(&self) -> &Arc<RwLock<Self::Metadata>>;

    /// Access metadata with read lock (sync closure)
    ///
    /// # Parameters
    /// - `f`: Synchronous closure that takes an immutable reference to metadata
    ///
    /// # Returns
    /// Returns the result produced by the closure
    ///
    /// # Example
    /// ```rust,ignore
    /// let server = exchange.with_metadata_read(|meta| {
    ///     meta.server_name().clone()
    /// }).await;
    /// ```
    async fn with_metadata_read<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a Self::Metadata) -> R + Send,
        R: Send,
    {
        let guard = self.metadata().read().await;
        f(&*guard)
    }

    /// Access metadata with write lock (sync closure)
    ///
    /// # Parameters
    /// - `f`: Synchronous closure that takes a mutable reference to metadata
    ///
    /// # Returns
    /// Returns the result produced by the closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_metadata_write(|meta| {
    ///     meta.set_enabled(true);
    /// }).await;
    /// ```
    async fn with_metadata_write<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut Self::Metadata) -> R + Send,
        R: Send,
    {
        let mut guard = self.metadata().write().await;
        f(&mut *guard)
    }

    /// Access metadata with read lock (async closure)
    ///
    /// # Parameters
    /// - `f`: Asynchronous closure that takes an immutable reference to metadata
    ///
    /// # Returns
    /// Returns the result produced by the async closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_metadata_read_async(|meta| {
    ///     Box::pin(async move {
    ///         meta.fetch_remote_config().await
    ///     })
    /// }).await;
    /// ```
    async fn with_metadata_read_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a Self::Metadata) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let guard = self.metadata().read().await;
        f(&*guard).await
    }

    /// Access metadata with write lock (async closure)
    ///
    /// # Parameters
    /// - `f`: Asynchronous closure that takes a mutable reference to metadata
    ///
    /// # Returns
    /// Returns the result produced by the async closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_metadata_write_async(|meta| {
    ///     Box::pin(async move {
    ///         meta.sync_with_remote().await;
    ///     })
    /// }).await;
    /// ```
    async fn with_metadata_write_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a mut Self::Metadata) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let mut guard = self.metadata().write().await;
        f(&mut *guard).await
    }
}

/// Processor accessor trait
///
/// Provides convenient methods to access exchange data processor with proper lock management
///
/// # Associated Types
/// - `Processor`: Exchange-specific data processor type, must implement `DataProcessor`
///
/// # Example
///
/// ```rust,ignore
/// impl ProcessorAccessor for Binance {
///     type Processor = BinanceDataProcessor;
///
///     fn processor(&self) -> &Arc<RwLock<Self::Processor>> {
///         &self.processor
///     }
/// }
///
/// // Read processor (sync closure)
/// let processor_name = exchange.with_processor_read(|processor| {
///     processor.processor_name().to_string()
/// }).await;
///
/// // Write processor (sync closure)
/// exchange.with_processor_write(|processor| {
///     processor.reset_state();
/// }).await;
///
/// // Read processor (async closure)
/// exchange.with_processor_read_async(|processor| {
///     Box::pin(async move {
///         processor.fetch_data().await
///     })
/// }).await;
///
/// // Write processor (async closure)
/// exchange.with_processor_write_async(|processor| {
///     Box::pin(async move {
///         processor.process_batch().await;
///     })
/// }).await;
/// ```
#[async_trait]
pub trait ProcessorAccessor: Send + Sync {
    /// Exchange-specific data processor type
    type Processor: DataProcessor;

    /// Get processor reference
    ///
    /// # Returns
    /// Returns an `Arc<RwLock<Processor>>` reference to the processor
    fn processor(&self) -> &Arc<RwLock<Self::Processor>>;

    /// Access processor with read lock (sync closure)
    ///
    /// # Parameters
    /// - `f`: Synchronous closure that takes an immutable reference to processor
    ///
    /// # Returns
    /// Returns the result produced by the closure
    ///
    /// # Example
    /// ```rust,ignore
    /// let name = exchange.with_processor_read(|processor| {
    ///     processor.name().clone()
    /// }).await;
    /// ```
    async fn with_processor_read<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a Self::Processor) -> R + Send,
        R: Send,
    {
        let guard = self.processor().read().await;
        f(&*guard)
    }

    /// Access processor with write lock (sync closure)
    ///
    /// # Parameters
    /// - `f`: Synchronous closure that takes a mutable reference to processor
    ///
    /// # Returns
    /// Returns the result produced by the closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_processor_write(|processor| {
    ///     processor.clear_cache();
    /// }).await;
    /// ```
    async fn with_processor_write<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut Self::Processor) -> R + Send,
        R: Send,
    {
        let mut guard = self.processor().write().await;
        f(&mut *guard)
    }

    /// Access processor with read lock (async closure)
    ///
    /// # Parameters
    /// - `f`: Asynchronous closure that takes an immutable reference to processor
    ///
    /// # Returns
    /// Returns the result produced by the async closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_processor_read_async(|processor| {
    ///     Box::pin(async move {
    ///         processor.fetch_remote_data().await
    ///     })
    /// }).await;
    /// ```
    async fn with_processor_read_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a Self::Processor) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let guard = self.processor().read().await;
        f(&*guard).await
    }

    /// Access processor with write lock (async closure)
    ///
    /// # Parameters
    /// - `f`: Asynchronous closure that takes a mutable reference to processor
    ///
    /// # Returns
    /// Returns the result produced by the async closure
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.with_processor_write_async(|processor| {
    ///     Box::pin(async move {
    ///         processor.process_batch().await;
    ///     })
    /// }).await;
    /// ```
    async fn with_processor_write_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a mut Self::Processor) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let mut guard = self.processor().write().await;
        f(&mut *guard).await
    }
}

// ============================================================================
// Exchange Lifecycle Trait
// ============================================================================

/// Exchange lifecycle management trait
///
/// Defines lifecycle operations for exchange (initialize, shutdown, state updates)
/// Depends on `MetadataAccessor` to access exchange metadata
///
/// # Associated Types
/// - `Error`: Exchange-specific error type, must implement `StarRiverErrorTrait`
///
/// # Example
///
/// ```rust,ignore
/// use exchange_core::{ExchangeLifecycle, MetadataAccessor};
///
/// #[async_trait]
/// impl ExchangeLifecycle for MetaTrader5 {
///     type Error = Mt5Error;
///
///     async fn initialize(&self) -> Result<(), Self::Error> {
///         let exchange_name = self.with_metadata_read(|meta| {
///             meta.server().to_string()
///         }).await;
///
///         tracing::info!("==========start exchange [{exchange_name}]==========");
///
///         // Transition to Initializing
///         self.update_state(ExchangeStateTransTrigger::StartInit).await?;
///
///         // Complete initialization
///         self.update_state(ExchangeStateTransTrigger::FinishInit).await?;
///
///         Ok(())
///     }
///
///     async fn shutdown(&self) -> Result<(), Self::Error> {
///         let exchange_name = self.with_metadata_read(|meta| {
///             meta.server().to_string()
///         }).await;
///
///         tracing::info!("==========stop exchange [{exchange_name}]==========");
///
///         // Transition to Stopping
///         self.update_state(ExchangeStateTransTrigger::Shutdown).await?;
///
///         // Complete shutdown
///         self.update_state(ExchangeStateTransTrigger::FinishShutdown).await?;
///
///         Ok(())
///     }
///
///     async fn update_state(&self, trigger: ExchangeStateTransTrigger) -> Result<(), Self::Error> {
///         let state_machine = self.state_machine().clone();
///
///         let transition_result = {
///             let mut sm = state_machine.write().await;
///             sm.transition(trigger)?
///         };
///
///         // Execute actions returned by state machine
///         for action in transition_result.actions() {
///             match action {
///                 Mt5Action::InitHttpClient => {
///                     // Initialize HTTP client
///                 }
///                 Mt5Action::InitWsClient => {
///                     // Initialize WebSocket client
///                 }
///                 // ... handle other actions
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait ExchangeLifecycle: MetadataAccessor {
    /// Exchange-specific error type
    type Error: StarRiverErrorTrait;

    /// Initialize the exchange
    ///
    /// Called before the exchange begins running to perform necessary initialization
    /// Specific exchanges can override to implement custom startup logic
    ///
    /// # Errors
    /// Returns error if initialization fails
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.initialize().await?;
    /// ```
    async fn initialize(&self) -> Result<(), Self::Error>;

    /// Shutdown the exchange
    ///
    /// Gracefully stop the exchange and clean up resources
    /// Specific exchanges can override to implement custom cleanup logic
    ///
    /// # Errors
    /// Returns error if shutdown fails
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.shutdown().await?;
    /// ```
    async fn shutdown(&self) -> Result<(), Self::Error>;

    /// Update exchange state
    ///
    /// Handle exchange state transition events
    ///
    /// # Parameters
    /// - `trigger`: State transition trigger
    ///
    /// # Errors
    /// Returns error if state transition is invalid
    ///
    /// # Example
    /// ```rust,ignore
    /// exchange.update_state(ExchangeStateTransTrigger::StartInit).await?;
    /// ```
    async fn update_state(&self, trans_trigger: ExchangeStateTransTrigger) -> Result<(), Self::Error>;
}


/// Main exchange trait
///
/// Combines all exchange functionality traits
///
/// All implementing types must also implement:
/// - `ExchangeMarketDataExt`: For market data operations
/// - `ExchangeSymbolExt`: For symbol operations
/// - `ExchangeAccountExt`: For account operations (optional, commented out)
///
/// # Example
///
/// ```rust,ignore
/// #[async_trait]
/// impl Exchange for Binance {
///     async fn exchange_type(&self) -> ExchangeType {
///         ExchangeType::Binance
///     }
///
///     async fn run_state(&self) -> ExchangeRunState {
///         self.current_state().await
///     }
///
///     async fn is_in_state(&self, state: &ExchangeRunState) -> bool {
///         self.current_state().await == *state
///     }
/// }
/// ```
#[async_trait]
pub trait Exchange:
    ExchangeMarketDataExt +
    ExchangeSymbolExt +
    // ExchangeAccountExt +
    Debug +
    'static
{
    /// Get the exchange type
    async fn exchange_type(&self) -> ExchangeType;

    /// Get the current run state
    async fn run_state(&self) -> ExchangeRunState;

    /// Check if the exchange is in a specific state
    async fn is_in_state(&self, state: &ExchangeRunState) -> bool;
}









// ============================================================================
// Exchange Feature Extension Traits
// ============================================================================

/// Market data extension trait
///
/// Defines market data query capabilities for each exchange
///
/// # Associated Types
/// - `Error`: Exchange-specific error type, must implement `StarRiverErrorTrait`
///
/// # Example
///
/// ```rust,ignore
/// // Binance implementation
/// impl ExchangeMarketDataExt for Binance {
///     type Error = BinanceError;
///
///     async fn kline_series(&self, symbol: &str, interval: KlineInterval, limit: u32)
///         -> Result<Vec<Kline>, Self::Error> {
///         // Implement Binance kline series fetching
///     }
///
///     async fn kline_history(&self, symbol: &str, interval: KlineInterval, time_range: TimeRange)
///         -> Result<Vec<Kline>, Self::Error> {
///         // Implement Binance kline history fetching
///     }
/// }
/// ```
#[async_trait]
pub trait ExchangeMarketDataExt {
    /// Exchange-specific error type
    type Error: StarRiverErrorTrait;

    /// Get kline series data
    ///
    /// # Parameters
    /// - `symbol`: Trading pair symbol
    /// - `interval`: Kline interval
    /// - `limit`: Number of klines to fetch
    ///
    /// # Returns
    /// Returns a vector of kline data
    async fn kline_series(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: u32,
    ) -> Result<Vec<Kline>, Self::Error>;

    /// Get historical kline data
    ///
    /// # Parameters
    /// - `symbol`: Trading pair symbol
    /// - `interval`: Kline interval
    /// - `time_range`: Time range for historical data
    ///
    /// # Returns
    /// Returns a vector of historical kline data
    async fn kline_history(
        &self,
        symbol: &str,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, Self::Error>;
}

/// Account management extension trait
///
/// Uses associated types to define exchange-specific account info types and error types
///
/// # Associated Types
/// - `AccountInfo`: Exchange-specific account info type, must implement `OriginalAccountInfo`
/// - `Error`: Exchange-specific error type, must implement `StarRiverErrorTrait`
///
/// # Example
///
/// ```rust,ignore
/// // Binance implementation
/// impl ExchangeAccountExt for Binance {
///     type AccountInfo = BinanceAccountInfo;
///     type Error = BinanceError;
///
///     async fn account_info(&self) -> Result<Self::AccountInfo, Self::Error> {
///         // Implement Binance account info fetching
///     }
/// }
///
/// // MT5 implementation
/// impl ExchangeAccountExt for MetaTrader5 {
///     type AccountInfo = MT5AccountInfo;
///     type Error = Mt5Error;
///
///     async fn account_info(&self) -> Result<Self::AccountInfo, Self::Error> {
///         // Implement MT5 account info fetching
///     }
/// }
/// ```
#[async_trait]
pub trait ExchangeAccountExt {
    /// Account info type
    ///
    /// Each exchange must define its own account info type that implements `OriginalAccountInfo`
    type AccountInfo: OriginalAccountInfo;

    /// Exchange-specific error type
    type Error: StarRiverErrorTrait;

    /// Get account info (returns concrete type)
    ///
    /// # Returns
    /// Returns the exchange-specific account info type
    ///
    /// # Benefits
    /// - Zero-cost abstraction, type determined at compile time
    /// - Type-safe, avoids runtime type conversion
    async fn account_info(&self) -> Result<Self::AccountInfo, Self::Error>;
}

/// Position management extension trait
///
/// Uses associated types to define exchange-specific position types and error types
///
/// # Associated Types
/// - `Position`: Exchange-specific position type, must implement `OriginalPosition`
/// - `Error`: Exchange-specific error type, must implement `StarRiverErrorTrait`
///
/// # Example
///
/// ```rust,ignore
/// // Binance implementation
/// impl ExchangePositionExt for Binance {
///     type Position = BinancePosition;
///     type Error = BinanceError;
///
///     async fn position(&self, params: GetPositionParam) -> Result<Self::Position, Self::Error> {
///         // Implement Binance position fetching
///     }
///
///     async fn position_number(&self, params: GetPositionNumberParams) -> Result<PositionNumber, Self::Error> {
///         // Implement Binance position number fetching
///     }
///
///     async fn latest_position(&self, position: &Position) -> Result<Position, Self::Error> {
///         // Implement latest position state fetching
///     }
/// }
///
/// // MT5 implementation
/// impl ExchangePositionExt for MetaTrader5 {
///     type Position = MT5Position;
///     type Error = Mt5Error;
///
///     async fn position(&self, params: GetPositionParam) -> Result<Self::Position, Self::Error> {
///         // Implement MT5 position fetching
///     }
/// }
/// ```
#[async_trait]
pub trait ExchangePositionExt {
    /// Position info type
    ///
    /// Each exchange must define its own position type that implements `OriginalPosition`
    type Position: OriginalPosition;

    /// Exchange-specific error type
    type Error: StarRiverErrorTrait;

    /// Get position details (returns concrete type)
    ///
    /// # Parameters
    /// - `params`: Position query parameters
    ///
    /// # Returns
    /// Returns the exchange-specific position type
    ///
    /// # Benefits
    /// - Zero-cost abstraction, type determined at compile time
    /// - Type-safe, avoids runtime type conversion
    async fn position(&self, params: GetPositionParam) -> Result<Self::Position, Self::Error>;

    /// Get position number info
    ///
    /// # Parameters
    /// - `params`: Position number query parameters
    ///
    /// # Returns
    /// Returns position number information
    async fn position_number(&self, params: GetPositionNumberParams) -> Result<PositionNumber, Self::Error>;

    /// Get latest position info
    ///
    /// # Parameters
    /// - `position`: Current position info
    ///
    /// # Returns
    /// Returns updated position information
    async fn latest_position(&self, position: &Position) -> Result<Position, Self::Error>;
}



/// Symbol management extension trait
///
/// Defines symbol query capabilities for each exchange
///
/// # Associated Types
/// - `Error`: Exchange-specific error type, must implement `StarRiverErrorTrait`
///
/// # Example
///
/// ```rust,ignore
/// impl ExchangeSymbolExt for Binance {
///     type Error = BinanceError;
///
///     async fn symbol_list(&self) -> Result<Vec<Symbol>, Self::Error> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait ExchangeSymbolExt {
    /// Exchange-specific error type
    type Error: StarRiverErrorTrait;

    /// Get list of all supported trading symbols
    async fn symbol_list(&self) -> Result<Vec<Symbol>, Self::Error>;

    /// Get information for a specific trading symbol
    async fn symbol(&self, symbol: String) -> Result<Symbol, Self::Error>;

    /// Get supported kline intervals
    fn support_kline_intervals(&self) -> Vec<KlineInterval>;
}