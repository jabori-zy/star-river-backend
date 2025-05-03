


use super::super::node_types::NodeRunState;
use super::super::NodeStateTransitionEvent;
use super::super::node_state_machine::{NodeStateMachine, StateChangeActions, TransitionAction};
use std::any::Any;

// 状态转换后需要执行的动作
#[derive(Debug, Clone)]
pub enum GetVariableNodeStateAction {
    ListenAndHandleExternalEvents,   // 处理外部事件
    ListenAndHandleMessage,         // 处理消息
    LogNodeState,    // 记录节点状态
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

impl TransitionAction for GetVariableNodeStateAction {
    fn get_action(&self) -> Box<dyn TransitionAction> {
        Box::new(self.clone())
    }
    fn clone_box(&self) -> Box<dyn TransitionAction> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct GetVariableNodeStateChangeActions {
    pub new_state: NodeRunState,
    pub actions: Vec<Box<dyn TransitionAction>>,
}

impl StateChangeActions for GetVariableNodeStateChangeActions {
    fn get_new_state(&self) -> NodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}


#[derive(Debug, Clone)]
pub struct GetVariableNodeStateMachine {
    current_state: NodeRunState,
    node_id: String,
    node_name: String,
}

impl GetVariableNodeStateMachine {
    pub fn new(node_id: String, node_name: String) -> Self {
        Self {
            current_state: NodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl NodeStateMachine for GetVariableNodeStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeStateMachine> {
        Box::new(self.clone())
    }

    // 获取当前状态
    fn current_state(&self) -> NodeRunState {
        self.current_state.clone()
    }

    fn transition(&mut self, event: NodeStateTransitionEvent) -> Result<Box<dyn StateChangeActions>, String> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (NodeRunState::Created, NodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Initializing;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Initializing,
                    actions: vec![
                        Box::new(GetVariableNodeStateAction::LogTransition), 
                        Box::new(GetVariableNodeStateAction::ListenAndHandleExternalEvents), 
                        Box::new(GetVariableNodeStateAction::ListenAndHandleMessage)],
                }))
            }
            // 初始化完成，进入Ready状态
            (NodeRunState::Initializing, NodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Ready;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Ready,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition), Box::new(GetVariableNodeStateAction::LogNodeState)],
                }))
            }
            // 从Ready状态开始启动
            (NodeRunState::Ready, NodeStateTransitionEvent::Start) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Starting;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Starting,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition)],
                }))
            }
            // 启动完成，进入Running状态
            (NodeRunState::Starting, NodeStateTransitionEvent::StartComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Running;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Running,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition)],
                }))
            }
            // 从Running状态开始停止
            (NodeRunState::Running, NodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopping;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Stopping,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition)],
                }))
            }
            // 停止完成，进入Stopped状态
            (NodeRunState::Stopping, NodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopped;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Stopped,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, NodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Failed;
                Ok(Box::new(GetVariableNodeStateChangeActions {
                    new_state: NodeRunState::Failed,
                    actions: vec![Box::new(GetVariableNodeStateAction::LogTransition), Box::new(GetVariableNodeStateAction::LogError(error))],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Failed;
                Err(format!("节点 {} 无效的状态转换: {:?} -> {:?}", self.node_id, state, event))
            }

        }
    }


}