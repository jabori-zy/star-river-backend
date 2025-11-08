use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use star_river_core::engine::EngineName;
use crate::state_machine::{EngineRunState, EngineAction, EngineStateMachine, EngineStateTransTrigger, StateChangeActions};
use event_center_new::{event::Event, communication::EngineCommand};

use crate::state_machine_error::EngineStateMachineError;
use super::context::EngineBaseContext;






// ============================================================================
// 引擎基础上下文 Trait
// ============================================================================

/// 引擎基础上下文 trait
///
/// 所有引擎上下文必须实现此 trait，提供对基础上下文的访问
///
/// # 关联类型
/// - `Action`: 引擎动作类型，必须实现 `EngineAction`
#[async_trait]
pub trait EngineContextTrait: Debug + Send + Sync + 'static {
    /// 引擎动作类型
    type Action: EngineAction;

    /// 获取基础上下文的不可变引用
    fn base_context(&self) -> &EngineBaseContext<Self::Action>;

    /// 获取基础上下文的可变引用
    fn base_context_mut(&mut self) -> &mut EngineBaseContext<Self::Action>;

    /// 获取引擎名称
    #[inline]
    fn engine_name(&self) -> &EngineName {
        self.base_context().engine_name()
    }

    /// 获取取消令牌
    #[inline]
    fn cancel_token(&self) -> &CancellationToken {
        self.base_context().cancel_token()
    }
}

// ============================================================================
// 扩展 Trait: EngineEventHandler - 事件处理（需要具体实现）
// ============================================================================

/// 引擎事件处理扩展
///
/// 定义引擎如何处理各种事件，需要具体引擎类型实现
#[async_trait]
pub trait EngineEventHandler: EngineContextTrait {
    /// 处理事件
    async fn handle_event(&mut self, event: Event);

    /// 处理命令
    async fn handle_command(&mut self, command: EngineCommand);
}

// ============================================================================
// 扩展 Trait: EngineStateMachineTrait - 状态机管理
// ============================================================================

/// 引擎状态机管理扩展
///
/// 管理引擎的运行状态和状态转换
#[async_trait]
pub trait EngineStateMachineTrait: EngineContextTrait {
    /// 获取状态机
    fn state_machine(&self) -> Arc<RwLock<EngineStateMachine<Self::Action>>> {
        self.base_context().state_machine()
    }

    /// 获取当前运行状态
    #[inline]
    async fn run_state(&self) -> EngineRunState {
        self.state_machine().read().await.current_state().clone()
    }

    /// 检查是否处于指定状态
    #[inline]
    async fn is_in_state(&self, state: &EngineRunState) -> bool {
        self.state_machine().read().await.is_in_state(state)
    }

    /// 状态转换
    #[inline]
    async fn transition_state(&self, trigger: EngineStateTransTrigger) -> Result<StateChangeActions<Self::Action>, EngineStateMachineError> {
        self.state_machine().write().await.transition(trigger)
    }
}

// 自动为所有实现 EngineContextTrait 的类型实现 EngineStateMachineTrait
impl<T> EngineStateMachineTrait for T
where
    T: EngineContextTrait,
{
}