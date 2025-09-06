use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;
use strum::Display;
use types::error::engine_error::strategy_engine_error::node_error::*;

// 状态转换后需要执行的动作
#[derive(Debug, Clone, Display)]
pub enum IndicatorNodeStateAction {
    #[strum(serialize = "ListenAndHandleExternalEvents")]
    ListenAndHandleExternalEvents,   // 处理外部事件
    #[strum(serialize = "ListenAndHandleNodeEvents")]
    ListenAndHandleNodeEvents, // 处理消息
    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 处理策略命令
    #[strum(serialize = "ListenAndHandleInnerEvents")]
    ListenAndHandleInnerEvents, // 处理内部事件
    #[strum(serialize = "RegisterIndicatorCacheKey")]
    RegisterIndicatorCacheKey, // 注册指标缓存键
    #[strum(serialize = "CalculateIndicator")]
    CalculateIndicator, // 计算指标
    #[strum(serialize = "LogNodeState")]
    LogNodeState,    // 记录节点状态
    #[strum(serialize = "LogTransition")]
    LogTransition,          // 记录状态转换
    #[strum(serialize = "LogError")]
    LogError(String),       // 记录错误
    #[strum(serialize = "CancelAsyncTask")]
    CancelAsyncTask,        // 取消异步任务
}

impl BacktestNodeTransitionAction for IndicatorNodeStateAction {
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
pub struct IndicatorNodeStateChangeActions {
    pub new_state: BacktestNodeRunState,
    pub actions: Vec<Box<dyn BacktestNodeTransitionAction>>,
}

impl BacktestStateChangeActions for IndicatorNodeStateChangeActions {
    fn get_new_state(&self) -> BacktestNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}

// 状态管理器
#[derive(Debug, Clone)]
pub struct IndicatorNodeStateManager {
    current_state: BacktestNodeRunState,
    node_id: String,
    node_name: String,
}

impl IndicatorNodeStateManager {
    pub fn new(current_state: BacktestNodeRunState, node_id: String, node_name: String) -> Self {
        Self {
            current_state,
            node_id,
            node_name,
        }
    }

}

impl BacktestNodeStateMachine for IndicatorNodeStateManager {
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
                Ok(Box::new(IndicatorNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Initializing,
                    actions: vec![
                        Box::new(IndicatorNodeStateAction::LogTransition), 
                        Box::new(IndicatorNodeStateAction::ListenAndHandleExternalEvents), 
                        Box::new(IndicatorNodeStateAction::ListenAndHandleNodeEvents),
                        Box::new(IndicatorNodeStateAction::ListenAndHandleStrategyCommand),
                        Box::new(IndicatorNodeStateAction::ListenAndHandleInnerEvents),
                        Box::new(IndicatorNodeStateAction::RegisterIndicatorCacheKey),
                        Box::new(IndicatorNodeStateAction::CalculateIndicator),
                        ],
                }))
            }
            // 初始化完成，进入Ready状态
            (BacktestNodeRunState::Initializing, BacktestNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Ready;
                Ok(Box::new(IndicatorNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Ready,
                    actions: vec![Box::new(IndicatorNodeStateAction::LogTransition), Box::new(IndicatorNodeStateAction::LogNodeState)],
                }))
            }
            // 从Running状态开始停止
            (BacktestNodeRunState::Ready, BacktestNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopping;
                Ok(Box::new(IndicatorNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopping,
                    actions: vec![
                        Box::new(IndicatorNodeStateAction::LogTransition),
                        Box::new(IndicatorNodeStateAction::CancelAsyncTask),
                    ],
                }))
            }
            // 停止完成，进入Stopped状态
            (BacktestNodeRunState::Stopping, BacktestNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopped;
                Ok(Box::new(IndicatorNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopped,
                    actions: vec![Box::new(IndicatorNodeStateAction::LogTransition), Box::new(IndicatorNodeStateAction::LogNodeState)],
                }))
            }
            // 从任何状态都可以失败
            (_, BacktestNodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Ok(Box::new(IndicatorNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Failed,
                    actions: vec![Box::new(IndicatorNodeStateAction::LogTransition), Box::new(IndicatorNodeStateAction::LogError(error))],
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