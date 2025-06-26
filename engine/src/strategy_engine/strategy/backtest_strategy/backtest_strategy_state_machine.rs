

#[derive(Debug, Clone, PartialEq)]
pub enum BacktestStrategyRunState { // 回测策略的运行状态
    Created,        // 策略已创建但未初始化
    Initializing,   // 策略正在初始化
    Ready,          // 策略已准备
    Stopping,       // 策略正在停止
    Stopped,        // 策略已停止
    Failed,         // 策略发生错误
}


#[derive(Debug)]
pub enum BacktestStrategyStateTransitionEvent { // 当切换到某一个状态时, 抛出的事件
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Stop,           // 停止策略
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 策略失败，带有错误信息
}





#[derive(Debug, Clone)]
pub enum BacktestStrategyStateAction { // 当切换到某一个状态时, 需要执行的动作
    InitCacheLength,      // 初始化缓存长度
    InitSignalCount,      // 初始化信号计数
    InitInitialPlaySpeed, // 初始化初始播放速度
    InitNode,             // 初始化节点
    StopNode,             // 停止节点
    ListenAndHandleNodeEvent,  // 监听节点消息
    ListenAndHandleCommand,  // 监听命令
    LogTransition,          // 记录状态转换
    LogError(String),       // 记录错误
}

#[derive(Debug)]
pub struct BacktestStrategyStateChangeActions { // 回测策略的状态转换动作
    pub new_state: BacktestStrategyRunState,
    pub actions: Vec<BacktestStrategyStateAction>,
}

impl BacktestStrategyStateChangeActions {
    fn get_new_state(&self) -> BacktestStrategyRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<BacktestStrategyStateAction> {
        self.actions.clone()
    }
}



#[derive(Debug, Clone)]
pub struct BacktestStrategyStateMachine {
    current_state: BacktestStrategyRunState,
    strategy_id: i32,
    strategy_name: String,
}

impl BacktestStrategyStateMachine {
    pub fn new(strategy_id: i32, strategy_name: String, current_state: BacktestStrategyRunState) -> Self {
        Self { current_state, strategy_id, strategy_name }
    }
}



impl BacktestStrategyStateMachine {
    pub fn current_state(&self) -> BacktestStrategyRunState {
        self.current_state.clone()
    }

    pub fn transition(&mut self, event: BacktestStrategyStateTransitionEvent) -> Result<BacktestStrategyStateChangeActions, String> {
        match (self.current_state.clone(), event) {
            // created => initializing
            (BacktestStrategyRunState::Created, BacktestStrategyStateTransitionEvent::Initialize) => {
                self.current_state = BacktestStrategyRunState::Initializing;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Initializing,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::ListenAndHandleNodeEvent,
                        BacktestStrategyStateAction::ListenAndHandleCommand,
                        BacktestStrategyStateAction::InitNode, // 初始化节点
                        BacktestStrategyStateAction::InitCacheLength, // 初始化缓存长度
                        BacktestStrategyStateAction::InitSignalCount, // 初始化信号计数
                        BacktestStrategyStateAction::InitInitialPlaySpeed, // 初始化初始播放速度
                        // BacktestStrategyStateAction::LoadPositions, // 加载持仓
                    ],
                })
            }
            // initializing => ready
            (BacktestStrategyRunState::Initializing, BacktestStrategyStateTransitionEvent::InitializeComplete) => {
                self.current_state = BacktestStrategyRunState::Ready;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Ready,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition
                    ],
                })
            }
            // running => stopping
            (BacktestStrategyRunState::Ready, BacktestStrategyStateTransitionEvent::Stop) => {
                self.current_state = BacktestStrategyRunState::Stopping;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Stopping,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition, 
                        BacktestStrategyStateAction::StopNode,
                    ],
                })
            }
            // stopping => stopped
            (BacktestStrategyRunState::Stopping, BacktestStrategyStateTransitionEvent::StopComplete) => {
                self.current_state = BacktestStrategyRunState::Stopped;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Stopped,
                    actions: vec![BacktestStrategyStateAction::LogTransition],
                })
            }
            // 从任何状态都可以失败
            (_, BacktestStrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = BacktestStrategyRunState::Failed;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Failed,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition, 
                        BacktestStrategyStateAction::LogError(error),
                    ],
                })
            }
            // 处理无效的状态转换
            (state, event) => {
                self.current_state = BacktestStrategyRunState::Failed;
                Err(format!("策略 {} 无效的状态转换: {:?} -> {:?}", self.strategy_name, state, event))
            }

        }
    }
}

