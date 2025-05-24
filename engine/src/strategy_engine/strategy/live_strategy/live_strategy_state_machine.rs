
#[derive(Debug, Clone, PartialEq)]
pub enum LiveStrategyRunState {
    Created,        // 策略已创建但未初始化
    Initializing,   // 策略正在初始化
    Ready,          // 策略已初始化，准备好但未运行
    Starting,       // 策略正在启动
    Running,        // 策略正在运行
    Stopping,       // 策略正在停止
    Stopped,        // 策略已停止
    Failed,         // 策略发生错误
}


#[derive(Debug)]
pub enum LiveStrategyStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动策略
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止策略
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 策略失败，带有错误信息
}

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


#[derive(Debug)]
pub struct LiveStrategyStateChangeActions {
    pub new_state: LiveStrategyRunState,
    pub actions: Vec<LiveStrategyStateAction>,
}

impl LiveStrategyStateChangeActions {
    pub fn get_new_state(&self) -> LiveStrategyRunState {
        self.new_state.clone()
    }
    pub fn get_actions(&self) -> Vec<LiveStrategyStateAction> {
        self.actions.clone()
    }
}



#[derive(Debug, Clone)]
pub struct LiveStrategyStateMachine {
    pub current_state: LiveStrategyRunState,
    pub strategy_id: i32,
    pub strategy_name: String,
}

impl LiveStrategyStateMachine {
    pub fn new(strategy_id: i32, strategy_name: String, current_state: LiveStrategyRunState) -> Self {
        Self { current_state, strategy_id, strategy_name }
    }
}



impl LiveStrategyStateMachine {

    pub fn current_state(&self) -> LiveStrategyRunState {
        self.current_state.clone()
    }

    pub fn transition(&mut self, event: LiveStrategyStateTransitionEvent) -> Result<LiveStrategyStateChangeActions, String> {
        match (self.current_state.clone(), event) {
            // created => initializing
            (LiveStrategyRunState::Created, LiveStrategyStateTransitionEvent::Initialize) => {
                self.current_state = LiveStrategyRunState::Initializing;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Initializing,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition,
                        LiveStrategyStateAction::ListenAndHandleEvent,
                        LiveStrategyStateAction::ListenAndHandleNodeMessage,
                        LiveStrategyStateAction::ListenAndHandleCommand,
                        LiveStrategyStateAction::InitNode,
                        LiveStrategyStateAction::LoadPositions,
                    ],
                })
            }
            // initializing => ready
            (LiveStrategyRunState::Initializing, LiveStrategyStateTransitionEvent::InitializeComplete) => {
                self.current_state = LiveStrategyRunState::Ready;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Ready,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition,
                    ],
                })
            }
            // ready => starting
            (LiveStrategyRunState::Ready, LiveStrategyStateTransitionEvent::Start) => {
                self.current_state = LiveStrategyRunState::Starting;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Starting,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition, 
                        LiveStrategyStateAction::StartNode,
                    ],
                })
            }
            // starting => running
            (LiveStrategyRunState::Starting, LiveStrategyStateTransitionEvent::StartComplete) => {
                self.current_state = LiveStrategyRunState::Running;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Running,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition,
                        LiveStrategyStateAction::RegisterTask, // 当启动成功之后, 注册任务
                    ],
                })
            }
            // running => stopping
            (LiveStrategyRunState::Running, LiveStrategyStateTransitionEvent::Stop) => {
                self.current_state = LiveStrategyRunState::Stopping;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Stopping,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition, 
                        LiveStrategyStateAction::StopNode,
                    ],
                })
            }
            // stopping => stopped
            (LiveStrategyRunState::Stopping, LiveStrategyStateTransitionEvent::StopComplete) => {
                self.current_state = LiveStrategyRunState::Stopped;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Stopped,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition,
                    ],
                })
            }
            // 从任何状态都可以失败
            (_, LiveStrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = LiveStrategyRunState::Failed;
                Ok(LiveStrategyStateChangeActions {
                    new_state: LiveStrategyRunState::Failed,
                    actions: vec![
                        LiveStrategyStateAction::LogTransition, 
                        LiveStrategyStateAction::LogError(error),
                    ],
                })
            }
            // 处理无效的状态转换
            (state, event) => {
                self.current_state = LiveStrategyRunState::Failed;
                Err(format!("策略 {} 无效的状态转换: {:?} -> {:?}", self.strategy_name, state, event))
            }

        }
    }
}

