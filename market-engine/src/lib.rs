mod context;
pub mod error;
mod lifecycle;
mod state_machine;
mod subkey;

use std::sync::Arc;

use context::MarketEngineContext;
use engine_core::{EngineBase, EngineContextAccessor, engine_trait::Engine};
use exchange_engine::ExchangeEngine;
use state_machine::MarketEngineAction;
use tokio::sync::{Mutex, RwLock};

use crate::error::MarketEngineError;

// ============================================================================
// MarketEngine struct (newtype pattern)
// ============================================================================

/// Market engine
#[derive(Debug)]
pub struct MarketEngine {
    inner: EngineBase<MarketEngineContext, MarketEngineAction, MarketEngineError>,
}

impl MarketEngine {
    /// Create a new market engine instance
    pub fn new(exchange_engine: Arc<Mutex<ExchangeEngine>>) -> Self {
        let context = MarketEngineContext::new(exchange_engine);

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref implementation - transparent access to inner EngineBase
// ============================================================================

impl std::ops::Deref for MarketEngine {
    type Target = EngineBase<MarketEngineContext, MarketEngineAction, MarketEngineError>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for MarketEngine {}

// ============================================================================
// EngineContextAccessor implementation - delegate to inner EngineBase
// ============================================================================

impl EngineContextAccessor for MarketEngine {
    type Context = MarketEngineContext;
    type Action = MarketEngineAction;
    type Error = MarketEngineError;

    fn context(&self) -> &Arc<RwLock<MarketEngineContext>> {
        self.inner.context()
    }
}
