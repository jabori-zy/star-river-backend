mod context;
pub mod error;
mod exchanges;
mod lifecycle;
mod state_machine;

use std::sync::Arc;

use context::ExchangeEngineContext;
use engine_core::{EngineBase, EngineBaseContext, EngineContextAccessor, engine_trait::Engine, state_machine::EngineRunState};
use sea_orm::DatabaseConnection;
use star_river_core::engine::EngineName;
use state_machine::ExchangeEngineAction;
use tokio::sync::RwLock;

use crate::state_machine::{ExchangeEngineStateMachine, exchange_engine_transition};

// ============================================================================
// ExchangeEngine 结构 (newtype 模式)
// ============================================================================

/// 交易所引擎
#[derive(Debug)]
pub struct ExchangeEngine {
    inner: EngineBase<ExchangeEngineContext, ExchangeEngineAction>,
}

impl ExchangeEngine {
    /// 创建新的交易所引擎实例
    pub fn new(database: DatabaseConnection) -> Self {
        let state_machine = ExchangeEngineStateMachine::new(
            EngineName::ExchangeEngine.to_string(),
            EngineRunState::Created,
            exchange_engine_transition,
        );
        let base_context = EngineBaseContext::new(EngineName::ExchangeEngine, state_machine);
        let context = ExchangeEngineContext::new(base_context, database);

        Self {
            inner: EngineBase::new(context),
        }
    }
}

// ============================================================================
// Deref 实现 - 透明访问内部 EngineBase
// ============================================================================

impl std::ops::Deref for ExchangeEngine {
    type Target = EngineBase<ExchangeEngineContext, ExchangeEngineAction>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Engine for ExchangeEngine {}

// ============================================================================
// EngineContextAccessor 实现 - 委托给内部 EngineBase
// ============================================================================

impl EngineContextAccessor for ExchangeEngine {
    type Context = ExchangeEngineContext;
    type Action = ExchangeEngineAction;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}
