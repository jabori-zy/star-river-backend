
use async_trait::async_trait;
use std::fmt::Debug;
use std::any::Any;
use super::node_types::*;

/// 状态管理器特征，定义所有状态管理器必须实现的方法
#[async_trait]
pub trait NodeStateMachine: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn NodeStateMachine>;
    /// 获取当前状态
    fn current_state(&self) -> NodeRunState;
    
    /// 处理状态转换事件
    fn transition(&mut self, event: NodeStateTransitionEvent) -> Result<Box<dyn StateChangeActions>, String>;

}


/// 状态转换后需要执行的动作
#[async_trait]
pub trait StateChangeActions: Debug + Send + Sync {
    fn get_new_state(&self) -> NodeRunState;
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>>;
}


/// 状态转换动作
pub trait TransitionAction: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn TransitionAction>;
    fn get_action(&self) -> Box<dyn TransitionAction>;
    fn as_any(&self) -> &dyn Any;
}



