
use async_trait::async_trait;
use std::fmt::Debug;
use std::any::Any;


/// 状态管理器特征，定义所有状态管理器必须实现的方法
#[async_trait]
pub trait LiveNodeStateMachine: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn LiveNodeStateMachine>;
    /// 获取当前状态
    fn current_state(&self) -> LiveNodeRunState;
    
    /// 处理状态转换事件
    fn transition(&mut self, event: LiveNodeStateTransitionEvent) -> Result<Box<dyn LiveStateChangeActions>, String>;

}


/// 状态转换后需要执行的动作
#[async_trait]
pub trait LiveStateChangeActions: Debug + Send + Sync {
    fn get_new_state(&self) -> LiveNodeRunState;
    fn get_actions(&self) -> Vec<Box<dyn LiveNodeTransitionAction>>;
}


/// 状态转换动作
pub trait LiveNodeTransitionAction: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn LiveNodeTransitionAction>;
    fn get_action(&self) -> Box<dyn LiveNodeTransitionAction>;
    fn as_any(&self) -> &dyn Any;
}




#[derive(Debug, Clone, PartialEq)]
pub enum LiveNodeRunState {
    Created,        // 节点已创建但未初始化
    Initializing,   // 节点正在初始化
    Ready,          // 节点已初始化，准备好但未运行
    Starting,      // 节点正在启动
    Running,        // 节点正在运行
    Stopping,       // 节点正在停止
    Stopped,        // 节点已停止
    Failed,         // 节点发生错误
}


// 状态转换事件
#[derive(Debug)]
pub enum LiveNodeStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动节点
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止节点
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 节点失败，带有错误信息
}




#[async_trait]
pub trait BacktestNodeStateMachine: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn BacktestNodeStateMachine>;
    /// 获取当前状态
    fn current_state(&self) -> BacktestNodeRunState;
    
    /// 处理状态转换事件
    fn transition(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<Box<dyn BacktestStateChangeActions>, String>;

}


/// 状态转换后需要执行的动作
#[async_trait]
pub trait BacktestStateChangeActions: Debug + Send + Sync {
    fn get_new_state(&self) -> BacktestNodeRunState;
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>>;
}


/// 状态转换动作
pub trait BacktestNodeTransitionAction: Debug + Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn BacktestNodeTransitionAction>;
    fn get_action(&self) -> Box<dyn BacktestNodeTransitionAction>;
    fn as_any(&self) -> &dyn Any;
}


#[derive(Debug, Clone, PartialEq)]
pub enum BacktestNodeRunState {
    Created,        // 节点已创建但未初始化
    Initializing,   // 节点正在初始化
    Ready,        // 节点已初始化，准备好但未运行
    Backtesting,    // 节点正在回测
    BacktestComplete,    // 节点回测完成
    Stopping,       // 节点正在停止
    Stopped,        // 节点已停止
    Failed,         // 节点发生错误
}


// 状态转换事件
#[derive(Debug)]
pub enum BacktestNodeStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Stop,           // 停止节点
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 节点失败，带有错误信息
}








