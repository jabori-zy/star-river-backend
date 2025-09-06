use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;
use strum::Display;
use types::error::engine_error::strategy_engine_error::node_error::*;

// 状态转换后需要执行的动作
#[derive(Debug, Clone, Display)]
pub enum PositionManagementNodeStateAction {
    #[strum(serialize = "ListenAndHandleExternalEvents")]
    ListenAndHandleExternalEvents,   // 处理外部事件
    #[strum(serialize = "ListenAndHandleNodeEvents")]
    ListenAndHandleNodeEvents,         // 处理消息
    #[strum(serialize = "ListenAndHandleInnerEvents")]
    ListenAndHandleInnerEvents,         // 处理内部事件
    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 处理策略命令
    #[strum(serialize = "ListenAndHandleVirtualTradingSystemEvent")]
    ListenAndHandleVirtualTradingSystemEvent, // 处理虚拟交易系统事件
    #[strum(serialize = "RegisterTask")]
    RegisterTask,        // 注册任务
    #[strum(serialize = "LogNodeState")]
    LogNodeState,    // 记录节点状态
    #[strum(serialize = "LogTransition")]
    LogTransition,          // 记录状态转换
    #[strum(serialize = "LogError")]
    LogError(String),       // 记录错误
}

impl BacktestNodeTransitionAction for PositionManagementNodeStateAction {
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
pub struct PositionNodeStateChangeActions {
    pub new_state: BacktestNodeRunState,
    pub actions: Vec<Box<dyn BacktestNodeTransitionAction>>,
}

impl BacktestStateChangeActions for PositionNodeStateChangeActions {
    fn get_new_state(&self) -> BacktestNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}


#[derive(Debug, Clone)]
pub struct PositionNodeStateMachine {
    current_state: BacktestNodeRunState,
    node_id: String,
    node_name: String,
}

impl PositionNodeStateMachine {
    pub fn new(node_id: String, node_name: String) -> Self {
        Self {
            current_state: BacktestNodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl BacktestNodeStateMachine for PositionNodeStateMachine {
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

    fn transition(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<Box<dyn BacktestStateChangeActions>, BacktestNodeStateMachineError> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (BacktestNodeRunState::Created, BacktestNodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Initializing;
                Ok(Box::new(PositionNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Initializing,
                    actions: vec![
                        Box::new(PositionManagementNodeStateAction::LogTransition), 
                        Box::new(PositionManagementNodeStateAction::ListenAndHandleExternalEvents), 
                        Box::new(PositionManagementNodeStateAction::ListenAndHandleNodeEvents),
                        Box::new(PositionManagementNodeStateAction::ListenAndHandleInnerEvents),
                        Box::new(PositionManagementNodeStateAction::ListenAndHandleStrategyCommand),
                        Box::new(PositionManagementNodeStateAction::ListenAndHandleVirtualTradingSystemEvent),
                        Box::new(PositionManagementNodeStateAction::RegisterTask)],
                }))
            }
            // 初始化完成，进入Ready状态
            (BacktestNodeRunState::Initializing, BacktestNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Ready;
                Ok(Box::new(PositionNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Ready,
                    actions: vec![Box::new(PositionManagementNodeStateAction::LogTransition), Box::new(PositionManagementNodeStateAction::LogNodeState)],
                }))
            }
            // 从Ready状态开始启动
            (BacktestNodeRunState::Ready, BacktestNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopping;
                Ok(Box::new(PositionNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopping,
                    actions: vec![Box::new(PositionManagementNodeStateAction::LogTransition)],
                }))
            }
            // 停止完成，进入Stopped状态
            (BacktestNodeRunState::Stopping, BacktestNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopped;
                Ok(Box::new(PositionNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopped,
                    actions: vec![Box::new(PositionManagementNodeStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, BacktestNodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Ok(Box::new(PositionNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Failed,
                    actions: vec![Box::new(PositionManagementNodeStateAction::LogTransition), Box::new(PositionManagementNodeStateAction::LogError(error))],
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
                }.fail()?
            }

        }
    }


}