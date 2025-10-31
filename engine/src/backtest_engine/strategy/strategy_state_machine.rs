use star_river_core::error::strategy_error::backtest_strategy_error::*;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum StrategyRunState {
    // 回测策略的运行状态
    #[strum(serialize = "Created")]
    Created, // 策略已创建但未初始化

    #[strum(serialize = "Checking")]
    Checking, // 策略正在检查

    #[strum(serialize = "CheckPassed")]
    CheckPassed, // 策略检查通过 -> 进入Created状态

    #[strum(serialize = "Initializing")]
    Initializing, // 策略正在初始化

    #[strum(serialize = "Ready")]
    Ready, // 策略已准备就绪

    #[strum(serialize = "Playing")]
    Playing, // 策略正在播放

    #[strum(serialize = "Pausing")]
    Pausing, // 策略正在暂停

    #[strum(serialize = "PlayComplete")]
    PlayComplete, // 策略播放完成

    #[strum(serialize = "Stopping")]
    Stopping, // 策略正在停止

    #[strum(serialize = "Stopped")]
    Stopped, // 策略已停止

    #[strum(serialize = "Failed")]
    Failed, // 策略发生错误
}

#[derive(Debug, Display)]
pub enum StrategyStateTransEvent {
    // 当切换到某一个状态时, 抛出的事件
    #[strum(serialize = "Check")]
    Check, // 检查策略
    #[strum(serialize = "CheckComplete")]
    CheckComplete, // 检查完成 -> 进入Created状态
    #[strum(serialize = "Initialize")]
    Initialize, // 初始化开始
    #[strum(serialize = "InitializeComplete")]
    InitializeComplete, // 初始化完成 -> 进入Ready状态
    #[strum(serialize = "Stop")]
    Stop, // 停止策略
    #[strum(serialize = "StopComplete")]
    StopComplete, // 停止完成 -> 进入Stopped状态
    #[strum(serialize = "Fail")]
    Fail(String), // 策略失败，带有错误信息
}

#[derive(Debug, Clone, Display)]
pub enum BacktestStrategyStateAction {
    // 当切换到某一个状态时, 需要执行的动作
    #[strum(serialize = "InitSignalCount")]
    InitSignalCount, // 初始化信号计数

    #[strum(serialize = "InitInitialPlaySpeed")]
    InitInitialPlaySpeed, // 初始化初始播放速度

    #[strum(serialize = "InitVirtualTradingSystem")]
    InitVirtualTradingSystem, // 初始化虚拟交易系统

    #[strum(serialize = "InitStrategyStats")]
    InitStrategyStats, // 初始化策略统计

    #[strum(serialize = "CheckNode")]
    CheckNode, // 检查节点

    #[strum(serialize = "InitNode")]
    InitNode, // 初始化节点

    #[strum(serialize = "StopNode")]
    StopNode, // 停止节点

    #[strum(serialize = "ListenAndHandleNodeEvent")]
    ListenAndHandleNodeEvent, // 监听节点消息

    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 监听命令

    #[strum(serialize = "ListenAndHandleStrategyStatsEvent")]
    ListenAndHandleStrategyStatsEvent, // 监听策略统计事件

    #[strum(serialize = "LogStrategyState")]
    LogStrategyState, // 记录策略状态

    #[strum(serialize = "LogTransition")]
    LogTransition, // 记录状态转换

    #[strum(serialize = "LogError")]
    LogError(String), // 记录错误
}

#[derive(Debug)]
pub struct BacktestStrategyStateChangeActions {
    // 回测策略的状态转换动作
    pub new_state: StrategyRunState,
    pub actions: Vec<BacktestStrategyStateAction>,
}

impl BacktestStrategyStateChangeActions {
    pub fn get_new_state(&self) -> StrategyRunState {
        self.new_state.clone()
    }

    pub fn get_actions(&self) -> Vec<BacktestStrategyStateAction> {
        self.actions.clone()
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
        Self {
            current_state,
            strategy_id,
            strategy_name,
        }
    }
}

impl BacktestStrategyStateMachine {
    pub fn current_state(&self) -> StrategyRunState {
        self.current_state.clone()
    }

    // 事件触发状态转换
    pub fn transition(
        &mut self,
        event: StrategyStateTransEvent,
    ) -> Result<BacktestStrategyStateChangeActions, BacktestStrategyError> {
        match (self.current_state.clone(), event) {
            (StrategyRunState::Created, StrategyStateTransEvent::Check) => {
                self.current_state = StrategyRunState::Checking;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Checking,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::CheckNode,
                    ],
                })
            }
            (StrategyRunState::Checking, StrategyStateTransEvent::CheckComplete) => {
                self.current_state = StrategyRunState::CheckPassed;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::CheckPassed,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                    ],
                })
            }
            // created => initializing
            (StrategyRunState::CheckPassed, StrategyStateTransEvent::Initialize) => {
                self.current_state = StrategyRunState::Initializing;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Initializing,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::ListenAndHandleNodeEvent,
                        BacktestStrategyStateAction::ListenAndHandleStrategyCommand,
                        BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent,
                        BacktestStrategyStateAction::InitNode,                 // 初始化节点
                        BacktestStrategyStateAction::InitSignalCount,          // 初始化信号计数
                        BacktestStrategyStateAction::InitInitialPlaySpeed,     // 初始化初始播放速度
                        BacktestStrategyStateAction::InitVirtualTradingSystem, // 初始化虚拟交易系统
                        BacktestStrategyStateAction::InitStrategyStats,        // 初始化策略统计
                    ],
                })
            }
            // initializing => ready
            (StrategyRunState::Initializing, StrategyStateTransEvent::InitializeComplete) => {
                self.current_state = StrategyRunState::Ready;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Ready,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                    ],
                })
            }
            // running => stopping
            (StrategyRunState::Ready, StrategyStateTransEvent::Stop) => {
                self.current_state = StrategyRunState::Stopping;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Stopping,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::StopNode,
                    ],
                })
            }
            // stopping => stopped
            (StrategyRunState::Stopping, StrategyStateTransEvent::StopComplete) => {
                self.current_state = StrategyRunState::Stopped;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Stopped,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                    ],
                })
            }
            // 从任何状态都可以失败
            (_, StrategyStateTransEvent::Fail(error)) => {
                self.current_state = StrategyRunState::Failed;
                Ok(BacktestStrategyStateChangeActions {
                    new_state: StrategyRunState::Failed,
                    actions: vec![
                        BacktestStrategyStateAction::LogTransition,
                        BacktestStrategyStateAction::LogStrategyState,
                        BacktestStrategyStateAction::LogError(error),
                    ],
                })
            }
            // 无效的（非法）状态转换
            (state, event) => {
                self.current_state = StrategyRunState::Failed;
                return Err(StrategyStateInvalidStateTransitionSnafu {
                    strategy_name: self.strategy_name.clone(),
                    current_state: state.to_string(),
                    event: event.to_string(),
                }
                .build());
            }
        }
    }
}
