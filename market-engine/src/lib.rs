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

// ============================================================================
// ExchangeEngine 结构 (newtype 模式)
// ============================================================================

/// 交易所引擎
#[derive(Debug)]
pub struct MarketEngine {
    inner: EngineBase<MarketEngineContext, MarketEngineAction>,
}

impl MarketEngine {
    /// 创建新的交易所引擎实例
    pub fn new(exchange_engine: Arc<Mutex<ExchangeEngine>>) -> Self {
        let context = MarketEngineContext::new(exchange_engine);

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref 实现 - 透明访问内部 EngineBase
// ============================================================================

impl std::ops::Deref for MarketEngine {
    type Target = EngineBase<MarketEngineContext, MarketEngineAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for MarketEngine {}

// ============================================================================
// EngineContextAccessor 实现 - 委托给内部 EngineBase
// ============================================================================

impl EngineContextAccessor for MarketEngine {
    type Context = MarketEngineContext;
    type Action = MarketEngineAction;

    fn context(&self) -> &Arc<RwLock<MarketEngineContext>> {
        self.inner.context()
    }
}
