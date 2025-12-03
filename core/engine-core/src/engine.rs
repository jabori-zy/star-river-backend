use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use star_river_core::error::StarRiverErrorTrait;
use tokio::sync::RwLock;

use crate::{context_trait::EngineContextTrait, engine_trait::EngineContextAccessor, state_machine::EngineAction};

// ============================================================================
// EngineBase 结构
// ============================================================================

/// 引擎基础结构
#[derive(Debug, Clone)]
pub struct EngineBase<C, Action, Error>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
    Error: StarRiverErrorTrait,
{
    /// 引擎上下文
    pub context: Arc<RwLock<C>>,
    _phantom: PhantomData<Error>,
}

impl<C, Action, Error> EngineBase<C, Action, Error>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
    Error: StarRiverErrorTrait,
{
    /// 创建新的引擎基础实例
    pub fn new(context: C) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
            _phantom: PhantomData,
        }
    }
}

impl<C, Action, Error> EngineContextAccessor for EngineBase<C, Action, Error>
where
    C: EngineContextTrait<Action = Action>,
    Action: EngineAction,
    Error: StarRiverErrorTrait,
{
    type Context = C;
    type Action = Action;
    type Error = Error;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}
