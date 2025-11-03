mod context;
mod subkey;
mod state_machine;
mod lifecycle;
pub mod error;



use context::MarketEngineContext;
use engine_core::engine_trait::Engine;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use engine_core::{EngineBase, EngineContextAccessor};
use state_machine::MarketEngineAction;
use exchange_engine::ExchangeEngine;

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

        let context = MarketEngineContext::new(
            exchange_engine
        );

        Self {
            inner: EngineBase::new(context)
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