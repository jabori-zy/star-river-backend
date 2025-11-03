use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::context_trait::EngineContextTrait;
use crate::engine_trait::EngineContextAccessor;
use crate::state_machine::EngineAction;

// ============================================================================
// EngineBase 结构
// ============================================================================

/// 引擎基础结构
#[derive(Debug, Clone)]
pub struct EngineBase<C, Action>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
{
    /// 引擎上下文
    pub context: Arc<RwLock<C>>,
}

impl<C, Action> EngineBase<C, Action>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
{
    /// 创建新的引擎基础实例
    pub fn new(context: C) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}


impl<C, Action> EngineContextAccessor for EngineBase<C, Action>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
{
    type Context = C;
    type Action = Action;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}

