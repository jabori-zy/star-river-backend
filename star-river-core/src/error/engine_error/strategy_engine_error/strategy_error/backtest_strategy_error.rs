use super::super::node_error::BacktestStrategyNodeError;
use crate::error::error_trait::Language;
use crate::error::ErrorCode;
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
// use event_center::EventCenterError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyError {
    #[snafu(display("backtest strategy node check error: {source}"))]
    NodeCheck {
        source: BacktestStrategyNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node init error: {source}"))]
    NodeInit {
        source: BacktestStrategyNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "backtest strategy node state not ready: {node_name}({node_id}) {node_type} node not ready"
    ))]
    NodeStateNotReady {
        node_id: String,
        node_name: String,
        node_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "backtest strategy node init timeout: {node_name}({node_id}) {node_type} node init timeout"
    ))]
    NodeInitTimeout {
        node_id: String,
        node_name: String,
        node_type: String,
        source: tokio::time::error::Elapsed,
        backtrace: Backtrace,
    },

    #[snafu(display("failed to execute {task_name} task for {node_name}({node_id}) {node_type}"))]
    TokioTaskFailed {
        task_name: String,
        node_name: String,
        node_id: String,
        node_type: String,
        source: tokio::task::JoinError,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}({strategy_id})] nodes config is null"))]
    NodeConfigNull {
        strategy_id: i32,
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}({strategy_id})] edges config is null"))]
    EdgeConfigNull {
        strategy_id: i32,
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "strategy [{strategy_name}({strategy_id})] edges config miss field: {field_name}"
    ))]
    EdgeConfigMissField {
        strategy_id: i32,
        strategy_name: String,
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}({strategy_id})] node not found"))]
    NodeNotFound {
        strategy_id: i32,
        strategy_name: String,
        node_id: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}({strategy_id})] invalid state transition: current state: {current_state}, event: {event}"))]
    StrategyStateInvalidStateTransition {
        strategy_id: i32,
        strategy_name: String,
        current_state: String,
        event: String,
        backtrace: Backtrace,
    },

    #[snafu(display("update strategy [{strategy_name}({strategy_id})] status failed: {source}"))]
    UpdateStrategyStatusFailed {
        strategy_id: i32,
        strategy_name: String,
        source: DbErr,
        backtrace: Backtrace,
    },

    #[snafu(display("wait for all nodes to stop timeout"))]
    WaitAllNodesStoppedTimeout { backtrace: Backtrace },

    #[snafu(display("all backtest data played finished"))]
    PlayFinished { backtrace: Backtrace },

    #[snafu(display("already playing, cannot play again"))]
    AlreadyPlaying { backtrace: Backtrace },

    #[snafu(display("already pausing, cannot pause again"))]
    AlreadyPausing { backtrace: Backtrace },


    #[snafu(display("different symbols have different minimum intervals: {symbols:?}"))]
    IntervalNotSame {
        symbols: Vec<(String, String)>,
        backtrace: Backtrace,

    },

    #[snafu(transparent)]
    Node {
        source: BacktestStrategyNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get data failed. key: {key}, play index: {play_index}"))]
    GetDataFailed {
        strategy_name: String,
        key: String,
        play_index: u32,
        backtrace: Backtrace,
    }


    // EventError {
    //     source: EventCenterError,
    //     backtrace: Backtrace,
    // },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl crate::error::error_trait::StarRiverErrorTrait for BacktestStrategyError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_STRATEGY"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            BacktestStrategyError::NodeCheck { .. } => 1000, // 节点检查错误
            BacktestStrategyError::NodeInit { .. } => 1001, // 节点初始化错误
            BacktestStrategyError::NodeInitTimeout { .. } => 1002, // 节点初始化超时
            BacktestStrategyError::TokioTaskFailed { .. } => 1003, // 执行任务失败
            BacktestStrategyError::NodeStateNotReady { .. } => 1004, // 节点状态未就绪
            BacktestStrategyError::NodeConfigNull { .. } => 1005, // 节点配置为空
            BacktestStrategyError::EdgeConfigNull { .. } => 1006, // 边配置为空
            BacktestStrategyError::EdgeConfigMissField { .. } => 1007, // 边配置缺少字段
            BacktestStrategyError::NodeNotFound { .. } => 1008, // 节点未找到
            BacktestStrategyError::StrategyStateInvalidStateTransition { .. } => 1009, // 策略状态转换无效
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => 1010, // 更新策略状态失败
            BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } => 1011, // 等待所有节点停止超时
            BacktestStrategyError::PlayFinished { .. } => 1012, // 所有回测数据播放完毕
            BacktestStrategyError::AlreadyPlaying { .. } => 1013, // 策略正在播放，无法再次播放
            BacktestStrategyError::AlreadyPausing { .. } => 1014, // 策略正在暂停，无法再次暂停
            BacktestStrategyError::IntervalNotSame { .. } => 1015, // 不同symbol的最小周期不相同
            BacktestStrategyError::Node { .. } => 1016, // 节点错误
            BacktestStrategyError::GetDataFailed { .. } => 1017, // 获取数据失败
            // BacktestStrategyError::EventSendError { .. } => 1010,
        };
        format!("{prefix}_{code}")
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            BacktestStrategyError::NodeInit { .. }
                | BacktestStrategyError::NodeInitTimeout { .. }
                | BacktestStrategyError::TokioTaskFailed { .. }
                | BacktestStrategyError::NodeStateNotReady { .. }
                | BacktestStrategyError::NodeConfigNull { .. }
                | BacktestStrategyError::EdgeConfigNull { .. }
                | BacktestStrategyError::EdgeConfigMissField { .. }
                | BacktestStrategyError::NodeNotFound { .. }
                | BacktestStrategyError::StrategyStateInvalidStateTransition { .. }
                | BacktestStrategyError::UpdateStrategyStatusFailed { .. }
                | BacktestStrategyError::WaitAllNodesStoppedTimeout { .. }
                | BacktestStrategyError::PlayFinished { .. }
                | BacktestStrategyError::AlreadyPlaying { .. }
                | BacktestStrategyError::AlreadyPausing { .. }
                | BacktestStrategyError::IntervalNotSame { .. }
                | BacktestStrategyError::Node { .. }
                | BacktestStrategyError::GetDataFailed { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => {
                match self {
                    BacktestStrategyError::NodeCheck { source, .. } => {
                        format!(
                            "回测策略节点检查错误: {}",
                            source.get_error_message(language)
                        )
                    }
                    BacktestStrategyError::NodeInit { source, .. } => {
                        format!(
                            "回测策略节点初始化错误: {}",
                            source.get_error_message(language)
                        )
                    }
                    BacktestStrategyError::NodeStateNotReady {
                        node_name,
                        node_id,
                        node_type,
                        ..
                    } => {
                        format!(
                            "回测策略节点状态未就绪: {}({}) {} 节点未准备好",
                            node_name, node_id, node_type
                        )
                    }
                    BacktestStrategyError::NodeInitTimeout {
                        node_name,
                        node_id,
                        node_type,
                        ..
                    } => {
                        format!(
                            "回测策略节点初始化超时: {}({}) {} 节点初始化超时",
                            node_name, node_id, node_type
                        )
                    }
                    BacktestStrategyError::TokioTaskFailed {
                        task_name,
                        node_name,
                        node_id,
                        node_type,
                        ..
                    } => {
                        format!(
                            "执行 {} 任务失败，节点: {}({}) {}",
                            task_name, node_name, node_id, node_type
                        )
                    }
                    BacktestStrategyError::NodeConfigNull {
                        strategy_name,
                        strategy_id,
                        ..
                    } => {
                        format!(
                            "节点配置为空: 策略 [{}({})] 节点配置为空",
                            strategy_name, strategy_id
                        )
                    }
                    BacktestStrategyError::EdgeConfigNull {
                        strategy_name,
                        strategy_id,
                        ..
                    } => {
                        format!(
                            "边配置为空: 策略 [{}({})] 边配置为空",
                            strategy_name, strategy_id
                        )
                    }
                    BacktestStrategyError::EdgeConfigMissField {
                        strategy_name,
                        strategy_id,
                        field_name,
                        ..
                    } => {
                        format!(
                            "边配置缺少字段: 策略 [{}({})] 边配置缺少字段: {}",
                            strategy_name, strategy_id, field_name
                        )
                    }
                    BacktestStrategyError::NodeNotFound {
                        strategy_name,
                        strategy_id,
                        node_id,
                        ..
                    } => {
                        format!(
                            "节点未找到: 策略 [{}({})] 节点 {} 未找到",
                            strategy_name, strategy_id, node_id
                        )
                    }
                    BacktestStrategyError::StrategyStateInvalidStateTransition {
                        strategy_name,
                        strategy_id,
                        current_state,
                        event,
                        ..
                    } => {
                        format!("策略状态转换无效: 策略 [{}({})] 无效的状态转换: 当前状态: {}, 事件: {}", strategy_name, strategy_id, current_state, event)
                    }
                    BacktestStrategyError::UpdateStrategyStatusFailed {
                        strategy_name,
                        strategy_id,
                        source,
                        ..
                    } => {
                        format!(
                            "更新策略状态失败: 策略 [{}({})] 更新状态失败: {}",
                            strategy_name, strategy_id, source
                        )
                    }
                    BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } => {
                        format!("等待所有节点停止超时")
                    }
                    BacktestStrategyError::PlayFinished { .. } => {
                        format!("所有回测数据播放完毕")
                    }
                    BacktestStrategyError::AlreadyPlaying { .. } => {
                        format!("策略正在播放，无法再次播放")
                    }
                    BacktestStrategyError::AlreadyPausing { .. } => {
                        format!("策略正在暂停，无法再次暂停")
                    }
                    BacktestStrategyError::IntervalNotSame { symbols, .. } => {
                        format!("不同交易对的最小周期不相同: {symbols:?}")
                    }
                    BacktestStrategyError::Node { source, .. } => {
                        format!("节点错误: {}", source.get_error_message(language))
                    }
                    BacktestStrategyError::GetDataFailed { strategy_name, key, play_index, .. } => {
                        format!("获取数据失败: 策略 [{strategy_name}] 数据键: {key}, 缓存索引: {play_index}")
                    }
                }
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // Errors with source - append self to source chain
            BacktestStrategyError::NodeCheck { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            BacktestStrategyError::NodeInit { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            BacktestStrategyError::NodeInitTimeout { .. } => {
                // tokio::time::error::Elapsed doesn't implement our trait, so we start the chain here
                vec![self.error_code()]
            }
            BacktestStrategyError::TokioTaskFailed { .. } => {
                // tokio::task::JoinError doesn't implement our trait, so we start the chain here
                vec![self.error_code()]
            }

            // Errors without source - use default implementation
            BacktestStrategyError::IntervalNotSame { .. } => {
                vec![self.error_code()]
            }
            BacktestStrategyError::Node { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            BacktestStrategyError::GetDataFailed { .. } => {
                vec![self.error_code()]
            }
            _ => vec![self.error_code()],
        }
    }
}
