use crate::strategy_engine::strategy::strategy_state_machine::*;
use std::any::Any;

#[derive(Debug, Clone)]
pub enum BacktestStrategyStateAction {
    InitNode,             // 初始化节点
    StartNode,            // 启动节点
    StopNode,             // 停止节点
    RegisterTask,      // 注册任务
    LoadPositions,        // 加载持仓
    ListenAndHandleNodeMessage,  // 监听节点消息
    ListenAndHandleCommand,  // 监听命令
    ListenAndHandleEvent,  // 监听事件消息
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

impl TransitionAction for BacktestStrategyStateAction {
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
    pub new_state: StrategyRunState,
    pub actions: Vec<Box<dyn TransitionAction>>,
}

impl StateChangeActions for StrategyStateChangeActions {
    fn get_new_state(&self) -> StrategyRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn TransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }
}



#[derive(Debug, Clone)]
pub struct BacktestStrategyStateMachine {
    current_state: StrategyRunState,
    strategy_id: i32,
    strategy_name: String,
}

impl BacktestStrategyStateMachine {
    pub fn new(strategy_id: i32, strategy_name: String, current_state: StrategyRunState) -> Self {
        Self { current_state, strategy_id, strategy_name }
    }
}



impl StrategyStateMachine for BacktestStrategyStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn StrategyStateMachine> {
        Box::new(self.clone())
    }

    fn current_state(&self) -> StrategyRunState {
        self.current_state.clone()
    }

    fn transition(&mut self, event: StrategyStateTransitionEvent) -> Result<Box<dyn StateChangeActions>, String> {
        match (self.current_state.clone(), event) {
            // created => initializing
            (StrategyRunState::Created, StrategyStateTransitionEvent::Initialize) => {
                self.current_state = StrategyRunState::Initializing;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Initializing,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition),
                        Box::new(BacktestStrategyStateAction::ListenAndHandleEvent),
                        Box::new(BacktestStrategyStateAction::ListenAndHandleNodeMessage),
                        Box::new(BacktestStrategyStateAction::ListenAndHandleCommand),
                        Box::new(BacktestStrategyStateAction::InitNode),
                        Box::new(BacktestStrategyStateAction::LoadPositions),
                    ],
                }))
            }
            // initializing => ready
            (StrategyRunState::Initializing, StrategyStateTransitionEvent::InitializeComplete) => {
                self.current_state = StrategyRunState::Ready;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Ready,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition)
                    ],
                }))
            }
            // ready => starting
            (StrategyRunState::Ready, StrategyStateTransitionEvent::Start) => {
                self.current_state = StrategyRunState::Starting;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Starting,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition), 
                        Box::new(BacktestStrategyStateAction::StartNode),
                    ],
                }))
            }
            // starting => running
            (StrategyRunState::Starting, StrategyStateTransitionEvent::StartComplete) => {
                self.current_state = StrategyRunState::Running;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Running,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition),
                        Box::new(BacktestStrategyStateAction::RegisterTask), // 当启动成功之后, 注册任务
                    ],
                }))
            }
            // running => stopping
            (StrategyRunState::Running, StrategyStateTransitionEvent::Stop) => {
                self.current_state = StrategyRunState::Stopping;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Stopping,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition), 
                        Box::new(BacktestStrategyStateAction::StopNode),
                    ],
                }))
            }
            // stopping => stopped
            (StrategyRunState::Stopping, StrategyStateTransitionEvent::StopComplete) => {
                self.current_state = StrategyRunState::Stopped;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Stopped,
                    actions: vec![Box::new(BacktestStrategyStateAction::LogTransition)],
                }))
            }
            // 从任何状态都可以失败
            (_, StrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = StrategyRunState::Failed;
                Ok(Box::new(StrategyStateChangeActions {
                    new_state: StrategyRunState::Failed,
                    actions: vec![
                        Box::new(BacktestStrategyStateAction::LogTransition), 
                        Box::new(BacktestStrategyStateAction::LogError(error)),
                    ],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                self.current_state = StrategyRunState::Failed;
                Err(format!("策略 {} 无效的状态转换: {:?} -> {:?}", self.strategy_name, state, event))
            }

        }
    }
}

