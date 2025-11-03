use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    exchange_trait::{DataProcessor, ExchangeMetadata, HttpClient, MetadataAccessor, ProcessorAccessor, WebSocketClient},
    state_machine::{ExchangeAction, ExchangeRunState, ExchangeStateMachine},
};

// ============================================================================
// ExchangeBase Structure
// ============================================================================

/// Exchange base structure
///
/// Uses generic parameterization to support custom components and metadata for different exchanges
///
/// # Generic Parameters
/// - `H`: HTTP client type, must implement `HttpClient`
/// - `W`: WebSocket client type, must implement `WebSocketClient`
/// - `P`: Data processor type, must implement `DataProcessor`
/// - `M`: Metadata type, must implement `ExchangeMetadata`
/// - `A`: Action type, must implement `ExchangeAction`
///
/// # Example
///
/// ```rust,ignore
/// // Define Binance actions
/// #[derive(Debug, Clone)]
/// pub enum BinanceAction {
///     Connect,
///     Disconnect,
///     Subscribe(String),
/// }
///
/// impl ExchangeAction for BinanceAction {}
///
/// // Define Binance exchange
/// pub type BinanceExchange = ExchangeBase<
///     BinanceHttpClient,
///     BinanceWsClient,
///     BinanceDataProcessor,
///     BinanceMetadata,
///     BinanceAction
/// >;
///
/// // Create instance with state machine
/// let state_machine = ExchangeStateMachine::new(
///     "binance".to_string(),
///     ExchangeRunState::Created,
///     default_exchange_transition,
/// );
///
/// let exchange = BinanceExchange::new(
///     http_client,
///     processor,
///     metadata,
///     state_machine,
/// );
/// ```
#[derive(Debug)]
pub struct ExchangeBase<H, W, P, M, A>
where
    H: HttpClient,
    W: WebSocketClient,
    P: DataProcessor,
    M: ExchangeMetadata,
    A: ExchangeAction,
{
    /// HTTP client (wrapped in Arc for multi-threaded sharing)
    http_client: Arc<H>,
    /// WebSocket client (wrapped in RwLock for concurrent read/write)
    ws_client: Arc<RwLock<Option<W>>>,
    /// Data processor (wrapped in RwLock for concurrent read/write)
    processor: Arc<RwLock<P>>,
    /// Exchange-specific metadata (wrapped in RwLock for concurrent read/write)
    metadata: Arc<RwLock<M>>,
    /// State machine for managing exchange lifecycle
    state_machine: Arc<RwLock<ExchangeStateMachine<A>>>,
}

impl<H, W, P, M, A> ExchangeBase<H, W, P, M, A>
where
    H: HttpClient,
    W: WebSocketClient,
    P: DataProcessor,
    M: ExchangeMetadata,
    A: ExchangeAction,
{
    /// Create a new exchange instance
    ///
    /// # Parameters
    /// - `http_client`: HTTP client instance
    /// - `processor`: Data processor instance
    /// - `metadata`: Metadata instance
    /// - `state_machine`: State machine for lifecycle management
    ///
    /// # Returns
    /// Returns a new `ExchangeBase` instance
    pub fn new(
        http_client: H,
        processor: P,
        metadata: M,
        state_machine: ExchangeStateMachine<A>,
    ) -> Self {
        Self {
            http_client: Arc::new(http_client),
            ws_client: Arc::new(RwLock::new(None)),
            processor: Arc::new(RwLock::new(processor)),
            metadata: Arc::new(RwLock::new(metadata)),
            state_machine: Arc::new(RwLock::new(state_machine)),
        }
    }

    /// Access HTTP client
    ///
    /// # Returns
    /// Returns an `Arc` reference to the HTTP client
    ///
    /// # Example
    /// ```rust,ignore
    /// let http_client = exchange.http_client();
    /// let base_url = http_client.base_url();
    /// ```
    #[inline]
    pub fn http_client(&self) -> &Arc<H> {
        &self.http_client
    }

    /// Access WebSocket client
    ///
    /// # Returns
    /// Returns an `Arc<RwLock>` reference to the WebSocket client
    ///
    /// # Example
    /// ```rust,ignore
    /// let ws_client = exchange.ws_client();
    /// let is_connected = ws_client.read().await.is_connected();
    /// ```
    #[inline]
    pub fn ws_client(&self) -> &Arc<RwLock<Option<W>>> {
        &self.ws_client
    }


    pub async fn set_ws_client(&self, ws_client: W) {
        let mut guard = self.ws_client.write().await;
        *guard = Some(ws_client);
    }

    /// Access data processor
    ///
    /// # Returns
    /// Returns an `Arc<RwLock>` reference to the data processor
    ///
    /// # Example
    /// ```rust,ignore
    /// let processor = exchange.processor();
    /// let name = processor.read().await.processor_name();
    /// ```
    #[inline]
    pub fn processor(&self) -> &Arc<RwLock<P>> {
        &self.processor
    }

    /// Access metadata
    ///
    /// # Returns
    /// Returns an `Arc<RwLock>` reference to the metadata
    ///
    /// # Example
    /// ```rust,ignore
    /// let metadata = exchange.metadata();
    /// let metadata_guard = metadata.read().await;
    /// // Access metadata fields
    /// ```
    #[inline]
    pub fn metadata(&self) -> &Arc<RwLock<M>> {
        &self.metadata
    }

    /// Access state machine
    ///
    /// # Returns
    /// Returns an `Arc<RwLock>` reference to the state machine
    ///
    /// # Example
    /// ```rust,ignore
    /// let state_machine = exchange.state_machine();
    /// let current_state = state_machine.read().await.current_state();
    /// ```
    #[inline]
    pub fn state_machine(&self) -> &Arc<RwLock<ExchangeStateMachine<A>>> {
        &self.state_machine
    }

    /// Get current exchange state
    ///
    /// # Returns
    /// Returns the current state of the exchange
    ///
    /// # Example
    /// ```rust,ignore
    /// let state = exchange.current_state().await;
    /// if state == ExchangeRunState::Running {
    ///     // Exchange is running
    /// }
    /// ```
    pub async fn current_state(&self) -> ExchangeRunState {
        *self.state_machine.read().await.current_state()
    }

    /// Check if exchange is in a specific state
    ///
    /// # Parameters
    /// - `state`: The state to check
    ///
    /// # Returns
    /// Returns true if the exchange is in the specified state
    pub async fn is_in_state(&self, state: &ExchangeRunState) -> bool {
        self.state_machine.read().await.is_in_state(state)
    }
}



// ============================================================================
// MetadataAccessor Implementation
// ============================================================================

impl<H, W, P, M, A> MetadataAccessor for ExchangeBase<H, W, P, M, A>
where
    H: HttpClient,
    W: WebSocketClient,
    P: DataProcessor,
    M: ExchangeMetadata,
    A: ExchangeAction,
{
    type Metadata = M;

    fn metadata(&self) -> &Arc<RwLock<Self::Metadata>> {
        &self.metadata
    }
}

// ============================================================================
// ProcessorAccessor Implementation
// ============================================================================

impl<H, W, P, M, A> ProcessorAccessor for ExchangeBase<H, W, P, M, A>
where
    H: HttpClient,
    W: WebSocketClient,
    P: DataProcessor,
    M: ExchangeMetadata,
    A: ExchangeAction,
{
    type Processor = P;

    fn processor(&self) -> &Arc<RwLock<Self::Processor>> {
        &self.processor
    }
}