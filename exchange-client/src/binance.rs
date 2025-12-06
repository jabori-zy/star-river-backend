mod binance_data_processor;
mod binance_http_client;
mod binance_type;
mod binance_ws_client;
mod client;
pub mod data_processor_error;
pub mod error;
mod lifecycle;
mod metadata;
mod state_machine;
mod url;
mod websocket;

// Re-export metadata for external use
use std::sync::Arc;

use async_trait::async_trait;
use exchange_core::{ExchangeBase, MetadataAccessor, ProcessorAccessor, state_machine::ExchangeRunState};
pub use metadata::BinanceMetadata;
use star_river_core::exchange::Exchange as ExchangeType;
use tokio::sync::RwLock;

use crate::binance::{
    binance_data_processor::BinanceDataProcessor,
    binance_http_client::BinanceHttpClient,
    binance_ws_client::BinanceWebSocket,
    state_machine::{BinanceAction, BinanceStateMachine, binance_transition},
};

// ============================================================================
// Binance Structure (newtype pattern)
// ============================================================================

/// Binance exchange client
///
/// Uses newtype pattern to wrap `ExchangeBase` and provide Binance-specific functionality
#[derive(Debug)]
pub struct Binance {
    inner: ExchangeBase<BinanceHttpClient, BinanceWebSocket, BinanceDataProcessor, BinanceMetadata, BinanceAction>,
}

impl Binance {
    /// Create a new Binance exchange instance
    ///
    /// # Parameters
    /// - `http_client`: Binance HTTP client
    /// - `ws_client`: Binance WebSocket client
    /// - `processor`: Binance data processor
    /// - `metadata`: Binance metadata
    ///
    /// # Returns
    /// Returns a new `Binance` instance
    pub fn new(metadata: BinanceMetadata) -> Self {
        let exchange = ExchangeType::Binance;
        let state_machine = BinanceStateMachine::new(exchange.to_string(), ExchangeRunState::Created, binance_transition);

        let http_client = BinanceHttpClient::new();
        let processor = BinanceDataProcessor {};

        Self {
            inner: ExchangeBase::new(http_client, processor, metadata, state_machine),
        }
    }
}

// ============================================================================
// Deref Implementation - Transparent access to inner ExchangeBase
// ============================================================================

impl std::ops::Deref for Binance {
    type Target = ExchangeBase<BinanceHttpClient, BinanceWebSocket, BinanceDataProcessor, BinanceMetadata, BinanceAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ============================================================================
// MetadataAccessor Implementation - Delegate to inner ExchangeBase
// ============================================================================

impl MetadataAccessor for Binance {
    type Metadata = BinanceMetadata;

    fn metadata(&self) -> &Arc<RwLock<Self::Metadata>> {
        self.inner.metadata()
    }
}

impl ProcessorAccessor for Binance {
    type Processor = BinanceDataProcessor;

    fn processor(&self) -> &Arc<RwLock<BinanceDataProcessor>> {
        self.inner.processor()
    }
}

#[async_trait]
impl exchange_core::exchange_trait::Exchange for Binance {
    async fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Binance
    }

    async fn run_state(&self) -> ExchangeRunState {
        self.state_machine().read().await.current_state().clone()
    }

    async fn is_in_state(&self, state: &ExchangeRunState) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }
}
