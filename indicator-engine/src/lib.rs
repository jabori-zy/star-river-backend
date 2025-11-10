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

// ============================================================================
// ExchangeEngine 结构 (newtype 模式)
// ============================================================================

/// 交易所引擎
#[derive(Debug)]
pub struct IndicatorEngine {
    inner: EngineBase<IndicatorEngineContext, IndicatorEngineAction>,
}

impl IndicatorEngine {
    /// 创建新的交易所引擎实例
    pub fn new() -> Self {
        let context = IndicatorEngineContext::new();

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref 实现 - 透明访问内部 EngineBase
// ============================================================================

impl std::ops::Deref for IndicatorEngine {
    type Target = EngineBase<IndicatorEngineContext, IndicatorEngineAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for IndicatorEngine {}

// ============================================================================
// EngineContextAccessor 实现 - 委托给内部 EngineBase
// ============================================================================

impl EngineContextAccessor for IndicatorEngine {
    type Context = IndicatorEngineContext;
    type Action = IndicatorEngineAction;
    fn context(&self) -> &Arc<RwLock<IndicatorEngineContext>> {
        self.inner.context()
    }
}
