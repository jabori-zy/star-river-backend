use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;

// 状态转换后需要执行的动作
#[derive(Debug, Clone)]
pub enum OrderNodeStateAction {
    ListenAndHandleExternalEvents,   // 处理外部事件
    ListenAndHandleMessage,         // 处理消息
    RegisterTask,          // 注册任务
    LogNodeState,    // 记录节点状态
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

impl LiveNodeTransitionAction for OrderNodeStateAction {
    fn get_action(&self) -> Box<dyn LiveNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn clone_box(&self) -> Box<dyn LiveNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug)]
pub struct OrderNodeStateChangeActions {
    pub new_state: LiveNodeRunState,
    pub actions: Vec<Box<dyn LiveNodeTransitionAction>>,
}

impl LiveStateChangeActions for OrderNodeStateChangeActions {
    fn get_new_state(&self) -> LiveNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn LiveNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}

// 状态管理器
#[derive(Debug, Clone)]
pub struct OrderNodeStateMachine {
    current_state: LiveNodeRunState,
    node_id: String,
    node_name: String,
}

impl OrderNodeStateMachine {
    pub fn new(node_id: String, node_name: String) -> Self {
        Self {
            current_state: LiveNodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl LiveNodeStateMachine for OrderNodeStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn LiveNodeStateMachine> {
        Box::new(self.clone())
    }

    // 获取当前状态
    fn current_state(&self) -> LiveNodeRunState {
        self.current_state.clone()
    }

    fn transition(&mut self, event: LiveNodeStateTransitionEvent) -> Result<Box<dyn LiveStateChangeActions>, String> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (LiveNodeRunState::Created, LiveNodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Initializing;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Initializing,
                    actions: vec![
                        Box::new(OrderNodeStateAction::LogTransition), 
                        Box::new(OrderNodeStateAction::ListenAndHandleExternalEvents), 
                        Box::new(OrderNodeStateAction::ListenAndHandleMessage),
                        Box::new(OrderNodeStateAction::RegisterTask)],
                }))
            }
            // 初始化完成，进入Ready状态
            (LiveNodeRunState::Initializing, LiveNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Ready;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Ready,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition), Box::new(OrderNodeStateAction::LogNodeState)],
                }))
            }
            // 从Ready状态开始启动
            (LiveNodeRunState::Ready, LiveNodeStateTransitionEvent::Start) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Starting;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Starting,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition)],
                }))
            }
            // 启动完成，进入Running状态
            (LiveNodeRunState::Starting, LiveNodeStateTransitionEvent::StartComplete) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Running;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Running,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition)],
                }))
            }
            // 从Running状态开始停止
            (LiveNodeRunState::Running, LiveNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Stopping;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Stopping,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition)],
                }))
            }
            // 停止完成，进入Stopped状态
            (LiveNodeRunState::Stopping, LiveNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Stopped;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Stopped,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, LiveNodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Failed;
                Ok(Box::new(OrderNodeStateChangeActions {
                    new_state: LiveNodeRunState::Failed,
                    actions: vec![Box::new(OrderNodeStateAction::LogTransition), Box::new(OrderNodeStateAction::LogError(error))],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = LiveNodeRunState::Failed;
                Err(format!("节点 {} 无效的状态转换: {:?} -> {:?}", self.node_id, state, event))
            }

        }
    }


}