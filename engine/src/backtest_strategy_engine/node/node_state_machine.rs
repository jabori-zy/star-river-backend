use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

use star_river_core::error::engine_error::strategy_engine_error::node_error::BacktestNodeStateMachineError;
use strum::Display;

#[async_trait]
pub trait BacktestNodeStateMachine: Send + Sync + Debug + 'static {
    fn as_any(&self) -> &dyn Any;

    fn clone_box(&self) -> Box<dyn BacktestNodeStateMachine>;
    /// 获取当前状态
    fn current_state(&self) -> BacktestNodeRunState;

    /// 处理状态转换事件
    fn transition(
        &mut self,
        event: BacktestNodeStateTransitionEvent,
    ) -> Result<Box<dyn BacktestStateChangeActions>, BacktestNodeStateMachineError>;
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

#[derive(Debug, Clone, PartialEq, Display)]
pub enum BacktestNodeRunState {
    #[strum(to_string = "Checking")]
    Checking, // 节点正在检查

    #[strum(to_string = "Created")]
    Created, // 节点已创建但未初始化

    #[strum(to_string = "Initializing")]
    Initializing, // 节点正在初始化

    #[strum(to_string = "Ready")]
    Ready, // 节点已初始化，准备好但未运行

    #[strum(to_string = "Backtesting")]
    Backtesting, // 节点正在回测

    #[strum(to_string = "BacktestComplete")]
    BacktestComplete, // 节点回测完成

    #[strum(to_string = "Stopping")]
    Stopping, // 节点正在停止

    #[strum(to_string = "Stopped")]
    Stopped, // 节点已停止

    #[strum(to_string = "Failed")]
    Failed, // 节点发生错误
}

// 状态转换事件
#[derive(Debug, Display)]
pub enum BacktestNodeStateTransitionEvent {
    #[strum(to_string = "Initialize")]
    Initialize, // 初始化开始
    #[strum(to_string = "InitializeComplete")]
    InitializeComplete, // 初始化完成 -> 进入Ready状态
    #[strum(to_string = "Stop")]
    Stop, // 停止节点
    #[strum(to_string = "StopComplete")]
    StopComplete, // 停止完成 -> 进入Stopped状态
    #[strum(to_string = "Failed")]
    Failed(String), // 节点失败，带有错误信息
}
