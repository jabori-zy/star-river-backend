use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;

// 状态转换后需要执行的动作
#[derive(Debug, Clone)]
pub enum OrderNodeStateAction {
    ListenAndHandleExternalEvents,   // 处理外部事件
    ListenAndHandleNodeEvents,         // 处理消息
    ListenAndHandleInnerEvents,         // 处理内部事件
    ListenAndHandleStrategyCommand, // 处理策略命令
    RegisterTask,          // 注册任务
    LogNodeState,    // 记录节点状态
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
    CancelAsyncTask,        // 取消异步任务
}

impl BacktestNodeTransitionAction for OrderNodeStateAction {
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
pub struct OrderNodeStateChangeActions {
    pub new_state: BacktestNodeRunState,
    pub actions: Vec<Box<dyn BacktestNodeTransitionAction>>,
}

impl BacktestStateChangeActions for OrderNodeStateChangeActions {
    fn get_new_state(&self) -> BacktestNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}

// 状态管理器
#[derive(Debug, Clone)]
pub struct OrderNodeStateMachine {
    current_state: BacktestNodeRunState,
    node_id: String,
    node_name: String,
}

impl OrderNodeStateMachine {
    pub fn new(node_id: String, node_name: String) -> Self {
        Self {
            current_state: BacktestNodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl BacktestNodeStateMachine for OrderNodeStateMachine {
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

    fn transition(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<Box<dyn BacktestStateChangeActions>, String> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (BacktestNodeRunState::Created, BacktestNodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Initializing;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Initializing,
                    actions: vec![
                        Box::new(OrderNodeStateAction::LogTransition), 
                        Box::new(OrderNodeStateAction::ListenAndHandleExternalEvents), 
                        Box::new(OrderNodeStateAction::ListenAndHandleNodeEvents),
                        Box::new(OrderNodeStateAction::ListenAndHandleInnerEvents),
                        Box::new(OrderNodeStateAction::ListenAndHandleStrategyCommand),
                        Box::new(OrderNodeStateAction::RegisterTask),
                    ],
                }))
            }
            // 初始化完成，进入Ready状态
            (BacktestNodeRunState::Initializing, BacktestNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Ready;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Ready,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition), Box::new(OrderNodeStateAction::LogNodeState)],
                }))
            }
            // 从Ready状态开始启动
            (BacktestNodeRunState::Ready, BacktestNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopping;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopping,
                    actions: vec![
                        Box::new(OrderNodeStateAction::LogTransition),
                        Box::new(OrderNodeStateAction::CancelAsyncTask),
                    ],
                }))
            }
            // 停止完成，进入Stopped状态
            (BacktestNodeRunState::Stopping, BacktestNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopped;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopped,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, BacktestNodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Failed,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition), Box::new(OrderNodeStateAction::LogError(error))],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Err(format!("节点 {} 无效的状态转换: {:?} -> {:?}", self.node_id, state, event))
            }

        }
    }


}