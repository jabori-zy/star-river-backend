use types::strategy::BacktestDataSource;
use types::error::engine_error::strategy_engine_error::node_error::*;
use strum::Display;

use crate::strategy_engine::node::node_state_machine::*;
use std::any::Any;


// 状态转换后需要执行的动作
#[derive(Debug, Clone, Display)]
pub enum KlineNodeStateAction {
    #[strum(serialize = "ListenAndHandleExternalEvents")]
    ListenAndHandleExternalEvents,   // 处理外部事件
    #[strum(serialize = "ListenAndHandleNodeEvents")]
    ListenAndHandleNodeEvents,    // 监听节点消息
    #[strum(serialize = "ListenAndHandleInnerEvents")]
    ListenAndHandleInnerEvents,    // 监听内部事件
    #[strum(serialize = "ListenAndHandleStrategyCommand")]
    ListenAndHandleStrategyCommand, // 处理策略命令
    LogNodeState,    // 记录节点状态
    #[strum(serialize = "RegisterExchange")]
    RegisterExchange, // 注册交易所
    #[strum(serialize = "LoadHistoryFromExchange")]
    LoadHistoryFromExchange, // 从交易所加载K线历史
    #[strum(serialize = "LoadHistoryFromFile")]
    LoadHistoryFromFile, // 从文件加载K线历史
    #[strum(serialize = "LogTransition")]
    LogTransition,          // 记录状态转换
    #[strum(serialize = "LogError")]
    LogError(String),       // 记录错误
    #[strum(serialize = "CancelAsyncTask")]
    CancelAsyncTask,        // 取消异步任务
}

impl BacktestNodeTransitionAction for KlineNodeStateAction {
    fn get_action(&self) -> Box<dyn BacktestNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeTransitionAction> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct KlineNodeStateChangeActions {
    pub new_state: BacktestNodeRunState,
    pub actions: Vec<Box<dyn BacktestNodeTransitionAction>>,
}

impl BacktestStateChangeActions for KlineNodeStateChangeActions {
    fn get_new_state(&self) -> BacktestNodeRunState {
        self.new_state.clone()
    }
    fn get_actions(&self) -> Vec<Box<dyn BacktestNodeTransitionAction>> {
        self.actions.iter().map(|action| action.clone_box()).collect()
    }


}


// 状态管理器
#[derive(Debug, Clone)]
pub struct KlineNodeStateMachine {
    data_source: BacktestDataSource, // 根据数据源的不同，执行不同的状态更新逻辑
    current_state: BacktestNodeRunState,
    node_id: String,
    node_name: String,
}

impl KlineNodeStateMachine {
    pub fn new(node_id: String, node_name: String, data_source: BacktestDataSource) -> Self {
        Self {
            data_source,
            current_state: BacktestNodeRunState::Created,
            node_id,
            node_name,
        }
    }
}

impl BacktestNodeStateMachine for KlineNodeStateMachine {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeStateMachine> {
        Box::new(self.clone())
    }
    
    // 获取当前状态
    fn current_state(&self) -> BacktestNodeRunState {
        self.current_state.clone()
    }

    fn transition(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<Box<dyn BacktestStateChangeActions>, BacktestNodeStateMachineError> {
        // 根据当前状态和事件确定新状态和需要执行的动作
        match (self.current_state.clone(), event) {
            // 从created状态开始初始化。执行初始化需要的方法
            (BacktestNodeRunState::Created, BacktestNodeStateTransitionEvent::Initialize) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Initializing;
                match self.data_source {
                    BacktestDataSource::Exchange => {
                        Ok(Box::new(KlineNodeStateChangeActions {
                            new_state: BacktestNodeRunState::Initializing,
                            actions: vec![
                                Box::new(KlineNodeStateAction::LogNodeState), // 节点状态日志
                                Box::new(KlineNodeStateAction::LogTransition),
                                Box::new(KlineNodeStateAction::ListenAndHandleExternalEvents),
                                Box::new(KlineNodeStateAction::ListenAndHandleNodeEvents),
                                Box::new(KlineNodeStateAction::ListenAndHandleInnerEvents),
                                Box::new(KlineNodeStateAction::ListenAndHandleStrategyCommand),
                                Box::new(KlineNodeStateAction::RegisterExchange), // 注册交易所
                                Box::new(KlineNodeStateAction::LoadHistoryFromExchange), // 从交易所加载K线历史
                            ],
                        }))

                    }
                    BacktestDataSource::File => {
                        Ok(Box::new(KlineNodeStateChangeActions {
                            new_state: BacktestNodeRunState::Initializing,
                            actions: vec![
                                Box::new(KlineNodeStateAction::LogNodeState), // 节点状态日志
                                Box::new(KlineNodeStateAction::LogTransition),
                                Box::new(KlineNodeStateAction::ListenAndHandleExternalEvents), 
                                Box::new(KlineNodeStateAction::LoadHistoryFromFile), // 从文件加载K线历史
                            ],
                        }))
                    }
                }
                
            }
            // 初始化完成，进入Ready状态
            (BacktestNodeRunState::Initializing, BacktestNodeStateTransitionEvent::InitializeComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Ready;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Ready,
                    actions: vec![
                        Box::new(KlineNodeStateAction::LogNodeState),
                        Box::new(KlineNodeStateAction::LogTransition), 
                        ],
                }))
            }
            // 从Running状态开始停止
            (BacktestNodeRunState::Ready, BacktestNodeStateTransitionEvent::Stop) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopping;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopping,
                    actions: vec![
                        Box::new(KlineNodeStateAction::LogNodeState),
                        Box::new(KlineNodeStateAction::LogTransition),
                        Box::new(KlineNodeStateAction::CancelAsyncTask),
                    ],
                }))
            }
            // 停止完成，进入Stopped状态
            (BacktestNodeRunState::Stopping, BacktestNodeStateTransitionEvent::StopComplete) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Stopped;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Stopped,
                    actions: vec![
                        Box::new(KlineNodeStateAction::LogNodeState),
                        Box::new(KlineNodeStateAction::LogTransition),
                    ],
                }))
            }
            // 从任何状态都可以失败
            (_, BacktestNodeStateTransitionEvent::Fail(error)) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                Ok(Box::new(KlineNodeStateChangeActions {
                    new_state: BacktestNodeRunState::Failed,
                    actions: vec![
                        Box::new(KlineNodeStateAction::LogNodeState),
                        Box::new(KlineNodeStateAction::LogTransition), 
                        Box::new(KlineNodeStateAction::LogError(error)),
                    ],
                }))
            }
            // 处理无效的状态转换
            (state, event) => {
                // 修改manager的状态
                self.current_state = BacktestNodeRunState::Failed;
                return NodeTransitionSnafu {
                    from_state: state.to_string(),
                    to_state: event.to_string(),
                    event: event.to_string(),
                }.fail()?
            }

        }
    }


}