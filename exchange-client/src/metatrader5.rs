mod client;
pub mod data_processor_error;
pub mod error;
mod metadata;
mod mt5_data_processor;
mod mt5_http_client;
mod mt5_types;
mod mt5_ws_client;
mod state_machine;
mod url;

// Re-export metadata for external use
use std::sync::Arc;

use async_trait::async_trait;
use exchange_core::{ExchangeBase, MetadataAccessor, ProcessorAccessor, state_machine::ExchangeRunState};
pub use metadata::Mt5Metadata;
use star_river_core::exchange::Exchange as ExchangeType;
use tokio::sync::RwLock;

use crate::metatrader5::{
    mt5_data_processor::Mt5DataProcessor,
    mt5_http_client::Mt5HttpClient,
    mt5_ws_client::Mt5WsClient,
    state_machine::{Mt5Action, Mt5StateMachine, metatrader5_transition},
};

// ============================================================================
// MetaTrader5 Structure (newtype pattern)
// ============================================================================

/// MetaTrader5 exchange client
///
/// Uses newtype pattern to wrap `ExchangeBase` and provide MT5-specific functionality
#[derive(Debug)]
pub struct MetaTrader5 {
    inner: ExchangeBase<Mt5HttpClient, Mt5WsClient, Mt5DataProcessor, Mt5Metadata, Mt5Action>,
}

impl MetaTrader5 {
    /// Create a new MetaTrader5 exchange instance
    ///
    /// # Parameters
    /// - `http_client`: MT5 HTTP client
    /// - `ws_client`: MT5 WebSocket client
    /// - `processor`: MT5 data processor
    /// - `metadata`: MT5 metadata
    /// - `state_machine`: MT5 state machine
    ///
    /// # Returns
    /// Returns a new `MetaTrader5` instance
    pub fn new(metadata: Mt5Metadata) -> Self {
        let exchange = ExchangeType::Metatrader5(metadata.server().to_string());
        let state_machine = Mt5StateMachine::new(exchange.to_string(), ExchangeRunState::Created, metatrader5_transition);
        let http_client = Mt5HttpClient::new(metadata.terminal_id());
        let processor = Mt5DataProcessor::new(metadata.server().to_string());
        Self {
            inner: ExchangeBase::new(http_client, processor, metadata, state_machine),
        }
    }
}

// ============================================================================
// Deref Implementation - Transparent access to inner ExchangeBase
// ============================================================================

impl std::ops::Deref for MetaTrader5 {
    type Target = ExchangeBase<Mt5HttpClient, Mt5WsClient, Mt5DataProcessor, Mt5Metadata, Mt5Action>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// ============================================================================
// MetadataAccessor Implementation - Delegate to inner ExchangeBase
// ============================================================================

impl MetadataAccessor for MetaTrader5 {
    type Metadata = Mt5Metadata;

    fn metadata(&self) -> &Arc<RwLock<Mt5Metadata>> {
        self.inner.metadata()
    }
}

impl ProcessorAccessor for MetaTrader5 {
    type Processor = Mt5DataProcessor;

    fn processor(&self) -> &Arc<RwLock<Mt5DataProcessor>> {
        self.inner.processor()
    }
}

#[async_trait]
impl exchange_core::exchange_trait::Exchange for MetaTrader5 {
    async fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Metatrader5(self.with_metadata_read(|meta| meta.server().to_string()).await)
    }

    async fn run_state(&self) -> ExchangeRunState {
        self.state_machine().read().await.current_state().clone()
    }

    async fn is_in_state(&self, state: &ExchangeRunState) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }
}
