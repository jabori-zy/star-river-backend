use crate::strategy_engine::node::node_state_machine::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::*;
use std::any::Any;
use strum::Display;

// 状态转换后需要执行的动作
#[derive(Debug, Clone, Display)]
pub enum VariableNodeStateAction {
    #[strum(serialize = "ListenAndHandleNodeEvents")]
    ListenAndHandleNodeEvents, // 处理消息
    #[strum(serialize = "ListenAndHandleStrategyInnerEvents")]
    ListenAndHandleStrategyInnerEvents, // 处理策略内部事件
    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 处理策略命令
    #[strum(serialize = "RegisterTask")]
    RegisterTask, // 注册任务
    #[strum(serialize = "LogNodeState")]
    LogNodeState, // 记录节点状态
    #[strum(serialize = "LogTransition")]
    LogTransition, // 记录状态转换
    #[strum(serialize = "LogError")]
    LogError(String), // 记录错误
    #[strum(serialize = "CancelAsyncTask")]
    CancelAsyncTask, // 取消异步任务
}

impl BacktestNodeTransitionAction for VariableNodeStateAction {
    fn get_action(&self) -> Box<dyn BacktestNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct VariableNodeStateChangeActions {
    pub new_state: BacktestNodeRunState,
    pub actions: Vec<Box<dyn BacktestNodeTransitionAction>>,
}

impl BacktestStateChangeActions for VariableNodeStateChangeActions {
    fn get_new_state(&self) -> BacktestNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct VariableNodeStateMachine {
    current_state: BacktestNodeRunState,
    node_id: String,
    node_name: String,
}

impl VariableNodeStateMachine {
    pub fn new(node_id: String, node_name: String) -> Self {
        Self {
            current_state: BacktestNodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl BacktestNodeStateMachine for VariableNodeStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeStateMachine> {
        Box::new(self.clone())
    }

    // 获取当前状态
    fn current_state(&self) -> BacktestNodeRunState {
        self.current_state.clone()
    }

    fn transition(
        &mut self,
        event: BacktestNodeStateTransitionEvent,
    ) -> Result<Box<dyn BacktestStateChangeActions>, BacktestNodeStateMachineError> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (BacktestNodeRunState::Created, BacktestNodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Initializing;
                Ok(Box::new(VariableNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Initializing,
                    actions: vec![
                        Box::new(VariableNodeStateAction::LogTransition),
                        Box::new(VariableNodeStateAction::ListenAndHandleNodeEvents),
                        Box::new(VariableNodeStateAction::ListenAndHandleStrategyInnerEvents),
                        Box::new(VariableNodeStateAction::ListenAndHandleStrategyCommand),
                    ],
                }))
            }
            // 初始化完成，进入Ready状态
            (BacktestNodeRunState::Initializing, BacktestNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Ready;
                Ok(Box::new(VariableNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Ready,
                    actions: vec![
                        Box::new(VariableNodeStateAction::LogTransition),
                        Box::new(VariableNodeStateAction::LogNodeState),
                    ],
                }))
            }
            // 从Ready状态开始启动
            (BacktestNodeRunState::Ready, BacktestNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopping;
                Ok(Box::new(VariableNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopping,
                    actions: vec![
                        Box::new(VariableNodeStateAction::LogTransition),
                        Box::new(VariableNodeStateAction::RegisterTask),
                        Box::new(VariableNodeStateAction::CancelAsyncTask),
                    ],
                }))
            }
            // 停止完成，进入Stopped状态
            (BacktestNodeRunState::Stopping, BacktestNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopped;
                Ok(Box::new(VariableNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopped,
                    actions: vec![Box::new(VariableNodeStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, BacktestNodeStateTransitionEvent::Failed(error)) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Ok(Box::new(VariableNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Failed,
                    actions: vec![
                        Box::new(VariableNodeStateAction::LogTransition),
                        Box::new(VariableNodeStateAction::LogError(error)),
                    ],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                return NodeTransitionSnafu {
                    from_state: state.to_string(),
                    to_state: event.to_string(),
                    event: event.to_string(),
                }
                .fail()?;
            }
        }
    }
}
