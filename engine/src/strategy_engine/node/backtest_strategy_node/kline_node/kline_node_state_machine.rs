use types::strategy::BacktestDataSource;

use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;


// 状态转换后需要执行的动作
#[derive(Debug, Clone)]
pub enum KlineNodeStateAction {
    ListenAndHandleExternalEvents,   // 处理外部事件
    ListenNodeMessage,    // 监听节点消息
    LogNodeState,    // 记录节点状态
    RegisterExchange, // 注册交易所
    LoadHistoryFromExchange, // 从交易所加载K线历史
    LoadHistoryFromFile, // 从文件加载K线历史
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

impl TransitionAction for KlineNodeStateAction {
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
pub struct KlineNodeStateChangeActions {
    pub new_state: NodeRunState,
    pub actions: Vec<Box<dyn TransitionAction>>,
}

impl StateChangeActions for KlineNodeStateChangeActions {
    fn get_new_state(&self) -> NodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }


}


// 状态管理器
#[derive(Debug, Clone)]
pub struct KlineNodeStateMachine {
    data_source: BacktestDataSource, // 根据数据源的不同，执行不同的状态更新逻辑
    current_state: NodeRunState,
    node_id: String,
    node_name: String,
}

impl KlineNodeStateMachine {
    pub fn new(node_id: String, node_name: String, data_source: BacktestDataSource) -> Self {
        Self {
            data_source,
            current_state: NodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl NodeStateMachine for KlineNodeStateMachine {
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
                match self.data_source {
                    BacktestDataSource::Exchange => {
                        Ok(Box::new(KlineNodeStateChangeActions {
                            new_state: NodeRunState::Initializing,
                            actions: vec![
                                Box::new(KlineNodeStateAction::LogTransition), 
                                Box::new(KlineNodeStateAction::ListenAndHandleExternalEvents),
                                Box::new(KlineNodeStateAction::ListenNodeMessage),
                                Box::new(KlineNodeStateAction::RegisterExchange), // 注册交易所
                                Box::new(KlineNodeStateAction::LoadHistoryFromExchange), // 从交易所加载K线历史
                            ],
                        }))

                    }
                    BacktestDataSource::File => {
                        Ok(Box::new(KlineNodeStateChangeActions {
                            new_state: NodeRunState::Initializing,
                            actions: vec![
                                Box::new(KlineNodeStateAction::LogTransition),
                                Box::new(KlineNodeStateAction::ListenAndHandleExternalEvents), 
                                Box::new(KlineNodeStateAction::LoadHistoryFromFile), // 从文件加载K线历史
                            ],
                        }))
                    }
                }
                
            }
            // 初始化完成，进入Ready状态
            (NodeRunState::Initializing, NodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Ready;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Ready,
                    actions: vec![Box::new(KlineNodeStateAction::LogTransition), Box::new(KlineNodeStateAction::LogNodeState)],
                }))
            }
            // 从Ready状态开始启动
            (NodeRunState::Ready, NodeStateTransitionEvent::Start) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Starting;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Starting,
                    actions: vec![
                        Box::new(KlineNodeStateAction::LogTransition),
                    ],
                }))
            }
            // 启动完成，进入Running状态
            (NodeRunState::Starting, NodeStateTransitionEvent::StartComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Running;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Running,
                    actions: vec![Box::new(KlineNodeStateAction::LogTransition), Box::new(KlineNodeStateAction::LogNodeState)],
                }))
            }
            // 从Running状态开始停止
            (NodeRunState::Running, NodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopping;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Stopping,
                    actions: vec![Box::new(KlineNodeStateAction::LogTransition)],
                }))
            }
            // 停止完成，进入Stopped状态
            (NodeRunState::Stopping, NodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Stopped;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Stopped,
                    actions: vec![Box::new(KlineNodeStateAction::LogTransition), Box::new(KlineNodeStateAction::LogNodeState)],
                }))
            }
            // 从任何状态都可以失败
            (_, NodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = NodeRunState::Failed;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: NodeRunState::Failed,
                    actions: vec![Box::new(KlineNodeStateAction::LogTransition), Box::new(KlineNodeStateAction::LogError(error))],
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