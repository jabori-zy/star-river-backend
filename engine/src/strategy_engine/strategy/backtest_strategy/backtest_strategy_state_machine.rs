use strum::Display;
use types::error::engine_error::strategy_engine_error::strategy_error::*;


#[derive(Debug, Clone, PartialEq, Display)]
pub enum BacktestStrategyRunState { // 回测策略的运行状态

    #[strum(serialize = "Created")]
    Created,        // 策略已创建但未初始化
    
    #[strum(serialize = "Checking")]
    Checking,       // 策略正在检查
    
    #[strum(serialize = "CheckPassed")]
    CheckPassed,        // 策略检查通过 -> 进入Created状态
    
    #[strum(serialize = "Initializing")]
    Initializing,   // 策略正在初始化

    #[strum(serialize = "Ready")]
    Ready,          // 策略已准备就绪

    #[strum(serialize = "Playing")]
    Playing,    // 策略正在播放

    #[strum(serialize = "Pausing")]
    Pausing,    // 策略正在暂停

    #[strum(serialize = "PlayComplete")]
    PlayComplete,    // 策略播放完成
    
    #[strum(serialize = "Stopping")]
    Stopping,       // 策略正在停止

    #[strum(serialize = "Stopped")]
    Stopped,        // 策略已停止

    #[strum(serialize = "Failed")]
    Failed,         // 策略发生错误
}


#[derive(Debug, Display)]
pub enum BacktestStrategyStateTransitionEvent { // 当切换到某一个状态时, 抛出的事件
    #[strum(serialize = "Check")]
    Check,            // 检查策略
    #[strum(serialize = "CheckComplete")]
    CheckComplete,    // 检查完成 -> 进入Created状态
    #[strum(serialize = "Initialize")]
    Initialize,     // 初始化开始
    #[strum(serialize = "InitializeComplete")]
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    #[strum(serialize = "Stop")]
    Stop,           // 停止策略
    #[strum(serialize = "StopComplete")]
    StopComplete,   // 停止完成 -> 进入Stopped状态
    #[strum(serialize = "Fail")]
    Fail(String),  // 策略失败，带有错误信息
}





#[derive(Debug, Clone, Display)]
pub enum BacktestStrategyStateAction { // 当切换到某一个状态时, 需要执行的动作
    #[strum(serialize = "InitCacheLength")]
    InitCacheLength,      // 初始化缓存长度

    #[strum(serialize = "InitSignalCount")]
    InitSignalCount,      // 初始化信号计数

    #[strum(serialize = "InitInitialPlaySpeed")]
    InitInitialPlaySpeed, // 初始化初始播放速度

    #[strum(serialize = "InitVirtualTradingSystem")]
    InitVirtualTradingSystem, // 初始化虚拟交易系统

    #[strum(serialize = "InitStrategyStats")]
    InitStrategyStats,    // 初始化策略统计

    #[strum(serialize = "CheckNode")]
    CheckNode,           // 检查节点

    #[strum(serialize = "InitNode")]
    InitNode,             // 初始化节点

    #[strum(serialize = "StopNode")]
    StopNode,             // 停止节点

    #[strum(serialize = "ListenAndHandleNodeEvent")]
    ListenAndHandleNodeEvent,  // 监听节点消息

    #[strum(serialize = "ListenAndHandleNodeCommand")]
    ListenAndHandleNodeCommand,  // 监听命令

    #[strum(serialize = "ListenAndHandleStrategyStatsEvent")]
    ListenAndHandleStrategyStatsEvent,  // 监听策略统计事件

    #[strum(serialize = "LogStrategyState")]
    LogStrategyState,          // 记录策略状态

    #[strum(serialize = "LogTransition")]
    LogTransition,          // 记录状态转换

    #[strum(serialize = "LogError")]
    LogError(String),       // 记录错误
}

#[derive(Debug)]
pub struct BacktestStrategyStateChangeActions { // 回测策略的状态转换动作
    pub new_state: BacktestStrategyRunState,
    pub actions: Vec<BacktestStrategyStateAction>,
}

impl BacktestStrategyStateChangeActions {
    pub fn get_new_state(&self) -> BacktestStrategyRunState {
        self.new_state.clone()
    }

    pub fn get_actions(&self) -> Vec<BacktestStrategyStateAction> {
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

    // 事件触发状态转换
    pub fn transition(&mut self, event: BacktestStrategyStateTransitionEvent) -> Result<BacktestStrategyStateChangeActions, BacktestStrategyError> {
        match (self.current_state.clone(), event) {
            (BacktestStrategyRunState::Created, BacktestStrategyStateTransitionEvent::Check) => {
                self.current_state = BacktestStrategyRunState::Checking;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Checking,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::CheckNode,
                    ],
                })


            }
            (BacktestStrategyRunState::Checking, BacktestStrategyStateTransitionEvent::CheckComplete) => {
                self.current_state = BacktestStrategyRunState::CheckPassed;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::CheckPassed,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                    ],
                })
            }
            // created => initializing
            (BacktestStrategyRunState::CheckPassed, BacktestStrategyStateTransitionEvent::Initialize) => {
                self.current_state = BacktestStrategyRunState::Initializing;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Initializing,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::ListenAndHandleNodeEvent,
                        BacktestStrategyStateAction::ListenAndHandleNodeCommand,
                        BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent,
                        BacktestStrategyStateAction::InitNode, // 初始化节点
                        BacktestStrategyStateAction::InitCacheLength, // 初始化缓存长度
                        BacktestStrategyStateAction::InitSignalCount, // 初始化信号计数
                        BacktestStrategyStateAction::InitInitialPlaySpeed, // 初始化初始播放速度
                        BacktestStrategyStateAction::InitVirtualTradingSystem, // 初始化虚拟交易系统
                        BacktestStrategyStateAction::InitStrategyStats, // 初始化策略统计
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
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
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
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::StopNode,
                    ],
                })
            }
            // stopping => stopped
            (BacktestStrategyRunState::Stopping, BacktestStrategyStateTransitionEvent::StopComplete) => {
                self.current_state = BacktestStrategyRunState::Stopped;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Stopped,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState
                        
                        ],
                })
            }
            // 从任何状态都可以失败
            (_, BacktestStrategyStateTransitionEvent::Fail(error)) => {
                self.current_state = BacktestStrategyRunState::Failed;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: BacktestStrategyRunState::Failed,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::LogError(error),
                    ],
                })
            }
            // 无效的（非法）状态转换
            (state, event) => {
                self.current_state = BacktestStrategyRunState::Failed;
                return Err(StrategyStateInvalidStateTransitionSnafu {
                    strategy_id: self.strategy_id,
                    strategy_name: self.strategy_name.clone(),
                    current_state: state.to_string(),
                    event: event.to_string(),
                }.build());
            }

        }
    }
}

