


// 节点运行状态
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyRunState {
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
pub enum StrategyStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动策略
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止策略
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 策略失败，带有错误信息
}

#[derive(Debug)]
pub struct StrategyStateChangeActions {
    pub new_state: StrategyRunState,
    pub actions: Vec<StrategyStateAction>,
}

#[derive(Debug, Clone)]
pub enum StrategyStateAction {
    InitNode,             // 初始化节点
    StartNode,            // 启动节点
    StopNode,             // 停止节点
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

#[derive(Debug, Clone)]
pub struct StrategyStateManager {
    current_state: StrategyRunState,
    strategy_id: i32,
    strategy_name: String,
}

impl StrategyStateManager {
    pub fn new(strategy_id: i32, strategy_name: String, current_state: StrategyRunState) -> Self {
        Self { current_state, strategy_id, strategy_name }
    }

    pub fn current_state(&self) -> StrategyRunState {
        self.current_state.clone()
    }

    pub fn transition(&mut self, event: StrategyStateTransitionEvent) -> Result<StrategyStateChangeActions, String> {
        match (self.current_state.clone(), event) {
            // created => initializing
            (StrategyRunState::Created, StrategyStateTransitionEvent::Initialize) => {
                self.current_state = StrategyRunState::Initializing;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Initializing,
                    actions: vec![StrategyStateAction::LogTransition, StrategyStateAction::InitNode],
                })
            }
            // initializing => ready
            (StrategyRunState::Initializing, StrategyStateTransitionEvent::InitializeComplete) => {
                self.current_state = StrategyRunState::Ready;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Ready,
                    actions: vec![StrategyStateAction::LogTransition],
                })
            }
            // ready => starting
            (StrategyRunState::Ready, StrategyStateTransitionEvent::Start) => {
                self.current_state = StrategyRunState::Starting;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Starting,
                    actions: vec![StrategyStateAction::LogTransition, StrategyStateAction::StartNode],
                })
            }
            // starting => running
            (StrategyRunState::Starting, StrategyStateTransitionEvent::StartComplete) => {
                self.current_state = StrategyRunState::Running;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Running,
                    actions: vec![StrategyStateAction::LogTransition],
                })
            }
            // running => stopping
            (StrategyRunState::Running, StrategyStateTransitionEvent::Stop) => {
                self.current_state = StrategyRunState::Stopping;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Stopping,
                    actions: vec![StrategyStateAction::LogTransition, StrategyStateAction::StopNode],
                })
            }
            // stopping => stopped
            (StrategyRunState::Stopping, StrategyStateTransitionEvent::StopComplete) => {
                self.current_state = StrategyRunState::Stopped;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Stopped,
                    actions: vec![StrategyStateAction::LogTransition],
                })
            }
            // 从任何状态都可以失败
            (_, StrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = StrategyRunState::Failed;
                Ok(StrategyStateChangeActions {
                    new_state: StrategyRunState::Failed,
                    actions: vec![StrategyStateAction::LogTransition, StrategyStateAction::LogError(error)],
                })
            }
            // 处理无效的状态转换
            (state, event) => {
                self.current_state = StrategyRunState::Failed;
                Err(format!("策略 {} 无效的状态转换: {:?} -> {:?}", self.strategy_name, state, event))
            }

        }
    }
}

