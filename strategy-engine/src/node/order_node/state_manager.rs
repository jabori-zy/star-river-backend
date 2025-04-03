use crate::NodeRunState;
use crate::node::NodeStateTransitionEvent;

// 状态转换后需要执行的动作
#[derive(Debug, Clone)]
pub enum OrderNodeStateAction {
    ListenAndHandleExternalEvents,   // 处理外部事件
    ListenAndHandleMessage,         // 处理消息
    LogNodeState,    // 记录节点状态
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

#[derive(Debug)]
pub struct OrderNodeStateChangeActions {
    pub new_state: NodeRunState,
    pub actions: Vec<OrderNodeStateAction>,
}


// 状态管理器
#[derive(Debug, Clone)]
pub struct OrderNodeStateManager {
    current_state: NodeRunState,
    node_id: String,
    node_name: String,
}

impl OrderNodeStateManager {
    pub fn new(current_state: NodeRunState, node_id: String, node_name: String) -> Self {
        Self {
            current_state,
            node_id,
            node_name,
        }
    }

    // 获取当前状态
    pub fn current_state(&self) -> NodeRunState {
        self.current_state.clone()
    }

    pub fn transition(&mut self, event: NodeStateTransitionEvent) -> Result<OrderNodeStateChangeActions, String> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (NodeRunState::Created, NodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Initializing;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Initializing,
                    actions: vec![OrderNodeStateAction::LogTransition, OrderNodeStateAction::ListenAndHandleExternalEvents, OrderNodeStateAction::ListenAndHandleMessage],
                })
            }
            // 初始化完成，进入Ready状态
            (NodeRunState::Initializing, NodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Ready;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Ready,
                    actions: vec![OrderNodeStateAction::LogTransition, OrderNodeStateAction::LogNodeState],
                })
            }
            // 从Ready状态开始启动
            (NodeRunState::Ready, NodeStateTransitionEvent::Start) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Starting;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Starting,
                    actions: vec![OrderNodeStateAction::LogTransition],
                })
            }
            // 启动完成，进入Running状态
            (NodeRunState::Starting, NodeStateTransitionEvent::StartComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Running;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Running,
                    actions: vec![OrderNodeStateAction::LogTransition],
                })
            }
            // 从Running状态开始停止
            (NodeRunState::Running, NodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopping;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Stopping,
                    actions: vec![OrderNodeStateAction::LogTransition],
                })
            }
            // 停止完成，进入Stopped状态
            (NodeRunState::Stopping, NodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopped;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Stopped,
                    actions: vec![OrderNodeStateAction::LogTransition],
                })
            }
            // 从任何状态都可以失败
            (_, NodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Failed;
                Ok(OrderNodeStateChangeActions {
                    new_state: NodeRunState::Failed,
                    actions: vec![OrderNodeStateAction::LogTransition, OrderNodeStateAction::LogError(error)],
                })
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Failed;
                Err(format!("节点 {} 无效的状态转换: {:?} -> {:?}", self.node_id, state, event))
            }

        }
    }

    // 应用状态转换结果
    // pub fn apply_transition(&mut self, result: StateTransitionResult) {
    //     self.current_state = result.new_state;
    // }


}