mod context;
// mod subkey;
mod calculate;
pub mod error;
mod indicator_engine_type;
mod lifecycle;
mod state_machine;

use std::sync::Arc;

use context::IndicatorEngineContext;
use engine_core::{EngineBase, EngineContextAccessor, engine_trait::Engine};
pub use star_river_core::kline::Kline;
use state_machine::IndicatorEngineAction;
pub use ta_lib::TALib;
use tokio::sync::RwLock;

use crate::error::IndicatorEngineError;

// ============================================================================
// IndicatorEngine structure (newtype pattern)
// ============================================================================

/// Indicator engine
#[derive(Debug)]
pub struct IndicatorEngine {
    inner: EngineBase<IndicatorEngineContext, IndicatorEngineAction, IndicatorEngineError>,
}

impl IndicatorEngine {
    /// Create a new indicator engine instance
    pub fn new() -> Self {
        let context = IndicatorEngineContext::new();

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref implementation - transparent access to inner EngineBase
// ============================================================================

impl std::ops::Deref for IndicatorEngine {
    type Target = EngineBase<IndicatorEngineContext, IndicatorEngineAction, IndicatorEngineError>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for IndicatorEngine {}

// ============================================================================
// EngineContextAccessor implementation - delegated to inner EngineBase
// ============================================================================

impl EngineContextAccessor for IndicatorEngine {
    type Context = IndicatorEngineContext;
    type Action = IndicatorEngineAction;
    type Error = IndicatorEngineError;
    fn context(&self) -> &Arc<RwLock<IndicatorEngineContext>> {
        self.inner.context()
    }
}
