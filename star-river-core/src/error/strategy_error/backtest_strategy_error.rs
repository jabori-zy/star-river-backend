use super::node_error::backtest_node_error::BacktestNodeError;
use crate::custom_type::NodeId;
use crate::error::ErrorCode;
use crate::error::error_trait::ErrorLanguage;
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
// use event_center::EventCenterError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyError {

    #[snafu(transparent)]
    BacktestNodeError {
        source: BacktestNodeError,
        backtrace: Backtrace,
    },


    #[snafu(display("backtest strategy node check failed: {source}"))]
    NodeCheckFailed {
        source: BacktestNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node init failed: {source}"))]
    NodeInitFailed {
        source: BacktestNodeError,
        backtrace: Backtrace,
    },


    #[snafu(display("backtest strategy node stop failed: {source}"))]
    NodeStopFailed {
        source: BacktestNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node state not ready: [{node_name}] node not ready"))]
    NodeStateNotReady {
        node_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node init timeout: [{node_name}] node init timeout"))]
    NodeInitTimeout {
        node_name: String,
        source: tokio::time::error::Elapsed,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node stop timeout: [{node_name}] node stop timeout"))]
    NodeStopTimeout {
        node_name: String,
        source: tokio::time::error::Elapsed,
        backtrace: Backtrace,
    },

    #[snafu(display("failed to execute {task_name} task for [{node_name}]"))]
    TokioTaskFailed {
        task_name: String,
        node_name: String,
        source: tokio::task::JoinError,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}] nodes config is null"))]
    NodeConfigNull {
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}] edges config is null"))]
    EdgeConfigNull {
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}] edges config miss field: {field_name}"))]
    EdgeConfigMissField {
        strategy_name: String,
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}] node [{node_id}] not found"))]
    NodeNotFound {
        strategy_name: String,
        node_id: NodeId,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy [{strategy_name}] invalid state transition: current state: {current_state}, event: {event}"))]
    StrategyStateInvalidStateTransition {
        strategy_name: String,
        current_state: String,
        event: String,
        backtrace: Backtrace,
    },

    #[snafu(display("update strategy [{strategy_name}] status failed: {source}"))]
    UpdateStrategyStatusFailed {
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

    

    #[snafu(display("strategy [{strategy_name}] get data failed. key: {key}, play index: {play_index}"))]
    GetDataFailed {
        strategy_name: String,
        key: String,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get data by timestamp failed. key: {key}, datetime: {datetime}"))]
    GetDataByDatetimeFailed {
        strategy_name: String,
        key: String,
        datetime: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] get start node config failed"))]
    GetStartNodeConfigFailed { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] kline data lengths are not all the same"))]
    KlineDataLengthNotSame { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("kline key not found: {kline_key}"))]
    KlineKeyNotFound { kline_key: String, backtrace: Backtrace },

    #[snafu(display("play index out of range, kline data length: {kline_data_length}, play index: {play_index}"))]
    PlayIndexOutOfRange {
        kline_data_length: u32,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("custom variable [{var_name}] not exists"))]
    CustomVariableNotExist { var_name: String },

    #[snafu(display("the update operation value of custom variable [{var_name}] is none "))]
    CustomVariableUpdateOperationValueIsNone { var_name: String },

    #[snafu(display(
        "unsupport variable operation: {operation} for custom variable [{var_name}] of type [{currrent_var_type}] to type [{operation_var_type}]"
    ))]
    UnSupportVariableOperation {
        var_name: String,
        currrent_var_type: String,
        operation_var_type: String,
        operation: String,
    },

    #[snafu(display("divide by zero for custom variable [{var_name}]"))]
    DivideByZero { var_name: String },

    #[snafu(display("node benchmark not found: {node_id}"))]
    NodeBenchmarkNotFound {
        node_id: NodeId,
        backtrace: Backtrace,
    },


    #[snafu(display("node cycle detected"))]
    NodeCycleDetected {
        backtrace: Backtrace,
    }
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
            BacktestStrategyError::BacktestNodeError { .. } => 1000,           // 节点检查错误
            BacktestStrategyError::NodeCheckFailed { .. } => 1001,           // 节点检查错误
            BacktestStrategyError::NodeInitFailed { .. } => 1002,            // 节点初始化错误
            BacktestStrategyError::NodeStopFailed { .. } => 1003,            // 节点停止错误
            BacktestStrategyError::NodeInitTimeout { .. } => 1004,     // 节点初始化超时
            BacktestStrategyError::NodeStopTimeout { .. } => 1005,     // 节点停止超时
            BacktestStrategyError::TokioTaskFailed { .. } => 1006,     // 执行任务失败
            BacktestStrategyError::NodeStateNotReady { .. } => 1007,   // 节点状态未就绪
            BacktestStrategyError::NodeConfigNull { .. } => 1008,      // 节点配置为空
            BacktestStrategyError::EdgeConfigNull { .. } => 1009,      // 边配置为空
            BacktestStrategyError::EdgeConfigMissField { .. } => 1010, // 边配置缺少字段
            BacktestStrategyError::NodeNotFound { .. } => 1011,        // 节点未找到
            BacktestStrategyError::StrategyStateInvalidStateTransition { .. } => 1012, // 策略状态转换无效
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => 1013, // 更新策略状态失败
            BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } => 1014, // 等待所有节点停止超时
            BacktestStrategyError::PlayFinished { .. } => 1015,        // 所有回测数据播放完毕
            BacktestStrategyError::AlreadyPlaying { .. } => 1016,      // 策略正在播放，无法再次播放
            BacktestStrategyError::AlreadyPausing { .. } => 1017,      // 策略正在暂停，无法再次暂停
            BacktestStrategyError::IntervalNotSame { .. } => 1018,     // 不同symbol的最小周期不相同
            BacktestStrategyError::GetDataFailed { .. } => 1019,       // 获取数据失败
            BacktestStrategyError::GetDataByDatetimeFailed { .. } => 1020, // 获取数据失败
            BacktestStrategyError::GetStartNodeConfigFailed { .. } => 1021, // 获取开始节点配置失败
            BacktestStrategyError::KlineDataLengthNotSame { .. } => 1022, // kline数据长度不相同
            BacktestStrategyError::KlineKeyNotFound { .. } => 1023,    // kline key未找到
            BacktestStrategyError::PlayIndexOutOfRange { .. } => 1024, // 播放索引超出范围
            BacktestStrategyError::CustomVariableNotExist { .. } => 1025, // 自定义变量不存在
            BacktestStrategyError::CustomVariableUpdateOperationValueIsNone { .. } => 1026, //变量的更新操作值为空
            BacktestStrategyError::UnSupportVariableOperation { .. } => 1027, // 不支持的变量操作
            BacktestStrategyError::DivideByZero { .. } => 1028,        // 除零错误
            BacktestStrategyError::NodeBenchmarkNotFound { .. } => 1029, // 节点benchmark未找到
            BacktestStrategyError::NodeCycleDetected { .. } => 1030, // 节点存在循环依赖
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
            BacktestStrategyError::NodeInitFailed { .. }
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
                | BacktestStrategyError::GetDataFailed { .. }
                | BacktestStrategyError::GetDataByDatetimeFailed { .. }
                | BacktestStrategyError::GetStartNodeConfigFailed { .. }
                | BacktestStrategyError::KlineDataLengthNotSame { .. }
                | BacktestStrategyError::KlineKeyNotFound { .. }
                | BacktestStrategyError::PlayIndexOutOfRange { .. }
                | BacktestStrategyError::NodeBenchmarkNotFound { .. }
        )
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BacktestStrategyError::BacktestNodeError { source, .. } => {
                    format!("回测节点错误: {}", source.error_message(language))
                }
                BacktestStrategyError::NodeCheckFailed { source, .. } => {
                    format!("回测节点检查错误: {}", source.error_message(language))
                }
                BacktestStrategyError::NodeInitFailed { source, .. } => {
                    format!("回测节点初始化错误: {}", source.error_message(language))
                }
                BacktestStrategyError::NodeStopFailed { source, .. } => {
                    format!("回测节点停止错误: {}", source.error_message(language))
                }
                BacktestStrategyError::NodeStopTimeout { node_name, .. } => {
                    format!("回测节点[{node_name}]停止超时")
                }
                BacktestStrategyError::NodeStateNotReady { node_name, .. } => {
                    format!("回测节点状态未就绪: [{node_name}] 节点未准备好")
                }
                BacktestStrategyError::NodeInitTimeout { node_name, .. } => {
                    format!("回测节点[{node_name}]初始化超时")
                }
                BacktestStrategyError::TokioTaskFailed { task_name, node_name, .. } => {
                    format!("执行 [{task_name}] 任务失败，节点: [{node_name}]")
                }
                BacktestStrategyError::NodeConfigNull {
                    strategy_name,
                    ..
                } => {
                    format!("节点配置为空: 策略 [{strategy_name}] 节点配置为空")
                }
                BacktestStrategyError::EdgeConfigNull {
                    strategy_name,
                    ..
                } => {
                    format!("边配置为空: 策略 [{strategy_name}] 边配置为空")
                }
                BacktestStrategyError::EdgeConfigMissField {
                    strategy_name,
                    field_name,
                    ..
                } => {
                    format!("边配置缺少字段: 策略 [{strategy_name}] 边配置缺少字段: {field_name}")
                }
                BacktestStrategyError::NodeNotFound {
                    strategy_name,
                    node_id: node_name,
                    ..
                } => {
                    format!("节点未找到: 策略 [{strategy_name}] 节点 [{node_name}] 未找到")
                }
                BacktestStrategyError::StrategyStateInvalidStateTransition {
                    strategy_name,
                    current_state,
                    event,
                    ..
                } => {
                    format!("策略状态转换无效: 策略 [{strategy_name}] 无效的状态转换: 当前状态: {current_state}, 事件: {event}")
                }
                BacktestStrategyError::UpdateStrategyStatusFailed {
                    strategy_name,
                    source,
                    ..
                } => {
                    format!("更新策略状态失败: 策略 [{strategy_name}] 更新状态失败: {source}")
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
                BacktestStrategyError::GetDataFailed {
                    strategy_name,
                    key,
                    play_index,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 获取数据失败. 数据键: {key}, 缓存索引: {play_index}")
                }
                BacktestStrategyError::GetDataByDatetimeFailed {
                    strategy_name,
                    key,
                    datetime,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 获取数据失败. 数据键: {key}, 时间: {datetime}")
                }
                BacktestStrategyError::GetStartNodeConfigFailed { strategy_name, .. } => {
                    format!("[{strategy_name}] 获取开始节点配置失败")
                }
                BacktestStrategyError::KlineDataLengthNotSame { strategy_name, .. } => {
                    format!("[{strategy_name}] kline数据长度不相同")
                }
                BacktestStrategyError::KlineKeyNotFound { kline_key, .. } => {
                    format!("kline key 不存在: {kline_key}")
                }
                BacktestStrategyError::PlayIndexOutOfRange {
                    kline_data_length,
                    play_index,
                    ..
                } => {
                    format!("播放索引超出范围, k线数据长度: {kline_data_length}, 播放索引: {play_index}")
                }

                BacktestStrategyError::CustomVariableNotExist { var_name } => {
                    format!("自定义变量[{var_name}]不存在.")
                }

                BacktestStrategyError::CustomVariableUpdateOperationValueIsNone { var_name } => {
                    format!("变量[{var_name}]的更新操作值为空")
                }
                BacktestStrategyError::UnSupportVariableOperation {
                    var_name,
                    currrent_var_type,
                    operation_var_type,
                    operation,
                    ..
                } => {
                    format!(
                        "不支持的变量操作: 自定义变量[{var_name}({currrent_var_type})] 不支持{operation}操作 to type [{operation_var_type}]"
                    )
                }

                BacktestStrategyError::DivideByZero { var_name } => {
                    format!("除零错误: 自定义变量[{var_name}]")
                }

                BacktestStrategyError::NodeBenchmarkNotFound { node_id, .. } => {
                    format!("节点benchmark未找到: {node_id}")
                }

                BacktestStrategyError::NodeCycleDetected { .. } => {
                    format!("节点存在循环依赖")
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BacktestStrategyError::BacktestNodeError { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            // Errors with source - append self to source chain
            BacktestStrategyError::NodeCheckFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            BacktestStrategyError::NodeInitFailed { source, .. } => {
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
            BacktestStrategyError::GetDataFailed { .. } => {
                vec![self.error_code()]
            }
            BacktestStrategyError::GetStartNodeConfigFailed { .. } => {
                vec![self.error_code()]
            }
            BacktestStrategyError::KlineDataLengthNotSame { .. } => {
                vec![self.error_code()]
            }
            BacktestStrategyError::KlineKeyNotFound { .. } => {
                vec![self.error_code()]
            }
            BacktestStrategyError::PlayIndexOutOfRange { .. } => {
                vec![self.error_code()]
            }
            _ => vec![self.error_code()],
        }
    }
}
