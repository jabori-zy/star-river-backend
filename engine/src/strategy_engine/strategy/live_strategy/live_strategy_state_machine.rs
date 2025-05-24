use crate::strategy_engine::strategy::strategy_state_machine::*;
use std::any::Any;

#[derive(Debug, Clone)]
pub enum LiveStrategyStateAction {
    InitNode,             // 初始化节点
    StartNode,            // 启动节点
    StopNode,             // 停止节点
    RegisterTask,      // 注册任务
    LoadPositions,        // 加载持仓
    ListenAndHandleNodeMessage,  // 监听节点消息
    ListenAndHandleEvent,  // 监听事件消息
    ListenAndHandleCommand,  // 监听命令
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

impl TransitionAction for LiveStrategyStateAction {
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
pub struct StrategyStateChangeActions {
    pub new_state: LiveStrategyRunState,
    pub actions: Vec<Box<dyn TransitionAction>>,
}

impl StateChangeActions for StrategyStateChangeActions {
    fn get_new_state(&self) -> LiveStrategyRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}



#[derive(Debug, Clone)]
pub struct LiveStrategyStateMachine {
    current_state: LiveStrategyRunState,
    strategy_id: i32,
    strategy_name: String,
}

impl LiveStrategyStateMachine {
    pub fn new(strategy_id: i32, strategy_name: String, current_state: LiveStrategyRunState) -> Self {
        Self { current_state, strategy_id, strategy_name }
    }
}



impl LiveStrategyStateMachineTrait for LiveStrategyStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn LiveStrategyStateMachineTrait> {
        Box::new(self.clone())
    }

    fn current_state(&self) -> LiveStrategyRunState {
        self.current_state.clone()
    }

    fn transition(&mut self, event: LiveStrategyStateTransitionEvent) -> Result<Box<dyn StateChangeActions>, String> {
        match (self.current_state.clone(), event) {
            // created => initializing
            (LiveStrategyRunState::Created, LiveStrategyStateTransitionEvent::Initialize) => {
                self.current_state = LiveStrategyRunState::Initializing;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Initializing,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition),
                        Box::new(LiveStrategyStateAction::ListenAndHandleEvent),
                        Box::new(LiveStrategyStateAction::ListenAndHandleNodeMessage),
                        Box::new(LiveStrategyStateAction::ListenAndHandleCommand),
                        Box::new(LiveStrategyStateAction::InitNode),
                        Box::new(LiveStrategyStateAction::LoadPositions),
                    ],
                }))
            }
            // initializing => ready
            (LiveStrategyRunState::Initializing, LiveStrategyStateTransitionEvent::InitializeComplete) => {
                self.current_state = LiveStrategyRunState::Ready;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Ready,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition)
                    ],
                }))
            }
            // ready => starting
            (LiveStrategyRunState::Ready, LiveStrategyStateTransitionEvent::Start) => {
                self.current_state = LiveStrategyRunState::Starting;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Starting,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition), 
                        Box::new(LiveStrategyStateAction::StartNode),
                    ],
                }))
            }
            // starting => running
            (LiveStrategyRunState::Starting, LiveStrategyStateTransitionEvent::StartComplete) => {
                self.current_state = LiveStrategyRunState::Running;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Running,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition),
                        Box::new(LiveStrategyStateAction::RegisterTask), // 当启动成功之后, 注册任务
                    ],
                }))
            }
            // running => stopping
            (LiveStrategyRunState::Running, LiveStrategyStateTransitionEvent::Stop) => {
                self.current_state = LiveStrategyRunState::Stopping;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Stopping,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition), 
                        Box::new(LiveStrategyStateAction::StopNode),
                    ],
                }))
            }
            // stopping => stopped
            (LiveStrategyRunState::Stopping, LiveStrategyStateTransitionEvent::StopComplete) => {
                self.current_state = LiveStrategyRunState::Stopped;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Stopped,
                    actions: vec![Box::new(LiveStrategyStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, LiveStrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = LiveStrategyRunState::Failed;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Failed,
                    actions: vec![
                        Box::new(LiveStrategyStateAction::LogTransition), 
                        Box::new(LiveStrategyStateAction::LogError(error)),
                    ],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                self.current_state = LiveStrategyRunState::Failed;
                Err(format!("策略 {} 无效的状态转换: {:?} -> {:?}", self.strategy_name, state, event))
            }

        }
    }
}

