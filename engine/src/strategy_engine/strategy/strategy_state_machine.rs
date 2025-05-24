use async_trait::async_trait;
use std::fmt::Debug;
use std::any::Any;


/// 状态管理器特征，定义所有状态管理器必须实现的方法
#[async_trait]
pub trait LiveStrategyStateMachineTrait: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn LiveStrategyStateMachineTrait>;
    /// 获取当前状态
    fn current_state(&self) -> LiveStrategyRunState;
    
    /// 处理状态转换事件
    fn transition(&mut self, event: LiveStrategyStateTransitionEvent) -> Result<Box<dyn StateChangeActions>, String>;

}


/// 状态转换后需要执行的动作
#[async_trait]
pub trait StateChangeActions: Debug + Send + Sync {
    fn get_new_state(&self) -> LiveStrategyRunState;
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>>;
}


/// 状态转换动作
pub trait TransitionAction: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn TransitionAction>;
    fn get_action(&self) -> Box<dyn TransitionAction>;
    fn as_any(&self) -> &dyn Any;
}

// 节点运行状态
#[derive(Debug, Clone, PartialEq)]
pub enum LiveStrategyRunState {
    Created,        // 策略已创建但未初始化
    Initializing,   // 策略正在初始化
    Ready,          // 策略已初始化，准备好但未运行
    Starting,       // 策略正在启动
    Running,        // 策略正在运行
    Stopping,       // 策略正在停止
    Stopped,        // 策略已停止
    Failed,         // 策略发生错误
}


#[derive(Debug)]
pub enum LiveStrategyStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动策略
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止策略
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 策略失败，带有错误信息
}