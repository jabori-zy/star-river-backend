use crate::error::node_error::BacktestNodeError;
use star_river_core::custom_type::NodeId;
use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode, generate_error_code_chain};
use star_river_core::strategy::strategy_benchmark::NodeBenchmarkNotFountError;
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
// use event_center::EventCenterError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyError {

    #[snafu(transparent)]
    BacktestNodeError {
        source: BacktestNodeError,
        backtrace: Backtrace,
    },


    #[snafu(transparent)]
    NodeBenchmarkNotFount {
        source: NodeBenchmarkNotFountError,
        backtrace: Backtrace,
    },


    #[snafu(display("[{strategy_name}] {node_name} node check failed: {source}"))]
    NodeCheckFailed {
        strategy_name: String,
        node_name: String,
        source: BacktestNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node init failed: {source}"))]
    NodeInitFailed {
        strategy_name: String,
        node_name: String,
        source: BacktestNodeError,
        backtrace: Backtrace,
    },


    #[snafu(display("[{strategy_name}] {node_name} node stop failed: {source}"))]
    NodeStopFailed {
        strategy_name: String,
        node_name: String,
        source: BacktestNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node state not ready: node not ready"))]
    NodeStateNotReady {
        strategy_name: String,
        node_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node init timeout: node init timeout"))]
    NodeInitTimeout {
        strategy_name: String,
        node_name: String,
        source: tokio::time::error::Elapsed,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node stop timeout: node stop timeout"))]
    NodeStopTimeout {
        strategy_name: String,
        node_name: String,
        source: tokio::time::error::Elapsed,
        backtrace: Backtrace,
    },

    #[snafu(display("failed to execute {task_name} task for [{strategy_name}] {node_name}"))]
    TokioTaskFailed {
        strategy_name: String,
        task_name: String,
        node_name: String,
        source: tokio::task::JoinError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] nodes config is null"))]
    NodeConfigNull {
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] edges config is null"))]
    EdgeConfigNull {
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] edges config miss field: {field_name}"))]
    EdgeConfigMissField {
        strategy_name: String,
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] node [{node_id}] not found"))]
    NodeNotFound {
        strategy_name: String,
        node_id: NodeId,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] invalid state transition: current state: {current_state}, trans trigger: {trans_trigger}"))]
    StrategyStateInvalidTrans {
        strategy_name: String,
        current_state: String,
        trans_trigger: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] update status failed: {source}"))]
    UpdateStrategyStatusFailed {
        strategy_name: String,
        source: DbErr,
        backtrace: Backtrace,
    },

    #[snafu(display("wait for all nodes to stop timeout"))]
    WaitAllNodesStoppedTimeout { backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] all backtest data played finished"))]
    PlayFinished { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("already playing, cannot play again"))]
    AlreadyPlaying { backtrace: Backtrace },

    #[snafu(display("already pausing, cannot pause again"))]
    AlreadyPausing { backtrace: Backtrace },

    #[snafu(display("different symbols have different minimum intervals: {symbols:?}"))]
    IntervalNotSame {
        symbols: Vec<(String, String)>,
        backtrace: Backtrace,
    },


    #[snafu(display("[{strategy_name}] get data failed. key: {key}, play index: {play_index}"))]
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

    #[snafu(display("[{strategy_name}] kline key not found: {kline_key}"))]
    KlineKeyNotFound {
        strategy_name: String,
        kline_key: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] play index out of range, kline data length: {kline_data_length}, play index: {play_index}"))]
    PlayIndexOutOfRange {
        strategy_name: String,
        kline_data_length: u32,
        play_index: u32,
        backtrace: Backtrace,
    },

    #[snafu(display("custom variable [{var_name}] not exists"))]
    CustomVariableNotExist {
        var_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("the update operation value of custom variable [{var_name}] is none "))]
    CusVarUpdateOpValueIsNone {
        var_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "unsupport variable operation: {operation} for custom variable [{var_name}] of type [{currrent_var_type}] to type [{operation_var_type}]"
    ))]
    UnSupportVariableOperation {
        var_name: String,
        currrent_var_type: String,
        operation_var_type: String,
        operation: String,
        backtrace: Backtrace,
    },

    #[snafu(display("divide by zero for custom variable [{var_name}]"))]
    DivideByZero {
        var_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] node benchmark not found: {node_name}"))]
    NodeBenchmarkNotFound {
        strategy_name: String,
        node_name: String,
        backtrace: Backtrace,
    },


    #[snafu(display("[{strategy_name}] node cycle detected"))]
    NodeCycleDetected {
        strategy_name: String,
        backtrace: Backtrace,
    }
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for BacktestStrategyError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_STRATEGY"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            BacktestStrategyError::BacktestNodeError { .. } => 1001,           // 节点检查错误
            BacktestStrategyError::NodeBenchmarkNotFount { .. } => 1002,       // 节点benchmark未找到
            BacktestStrategyError::NodeCheckFailed { .. } => 1003,           // 节点检查错误
            BacktestStrategyError::NodeInitFailed { .. } => 1004,            // 节点初始化错误
            BacktestStrategyError::NodeStopFailed { .. } => 1005,            // 节点停止错误
            BacktestStrategyError::NodeInitTimeout { .. } => 1006,     // 节点初始化超时
            BacktestStrategyError::NodeStopTimeout { .. } => 1007,     // 节点停止超时
            BacktestStrategyError::TokioTaskFailed { .. } => 1008,     // 执行任务失败
            BacktestStrategyError::NodeStateNotReady { .. } => 1009,   // 节点状态未就绪
            BacktestStrategyError::NodeConfigNull { .. } => 1010,      // 节点配置为空
            BacktestStrategyError::EdgeConfigNull { .. } => 1011,      // 边配置为空
            BacktestStrategyError::EdgeConfigMissField { .. } => 1012, // 边配置缺少字段
            BacktestStrategyError::NodeNotFound { .. } => 1013,        // 节点未找到
            BacktestStrategyError::StrategyStateInvalidTrans { .. } => 1014, // 策略状态转换无效
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => 1015, // 更新策略状态失败
            BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } => 1016, // 等待所有节点停止超时
            BacktestStrategyError::PlayFinished { .. } => 1017,        // 所有回测数据播放完毕
            BacktestStrategyError::AlreadyPlaying { .. } => 1018,      // 策略正在播放，无法再次播放
            BacktestStrategyError::AlreadyPausing { .. } => 1019,      // 策略正在暂停，无法再次暂停
            BacktestStrategyError::IntervalNotSame { .. } => 1020,     // 不同symbol的最小周期不相同
            BacktestStrategyError::GetDataFailed { .. } => 1021,       // 获取数据失败
            BacktestStrategyError::GetDataByDatetimeFailed { .. } => 1022, // 获取数据失败
            BacktestStrategyError::GetStartNodeConfigFailed { .. } => 1023, // 获取开始节点配置失败
            BacktestStrategyError::KlineDataLengthNotSame { .. } => 1024, // kline数据长度不相同
            BacktestStrategyError::KlineKeyNotFound { .. } => 1025,    // kline key未找到
            BacktestStrategyError::PlayIndexOutOfRange { .. } => 1026, // 播放索引超出范围
            BacktestStrategyError::CustomVariableNotExist { .. } => 1027, // 自定义变量不存在
            BacktestStrategyError::CusVarUpdateOpValueIsNone { .. } => 1028, // custom variable update operation value is none
            BacktestStrategyError::UnSupportVariableOperation { .. } => 1029, // 不支持的变量操作
            BacktestStrategyError::DivideByZero { .. } => 1030,        // 除零错误
            BacktestStrategyError::NodeBenchmarkNotFound { .. } => 1031, // 节点benchmark未找到
            BacktestStrategyError::NodeCycleDetected { .. } => 1032, // 节点存在循环依赖
        };
        format!("{prefix}_{code}")
    }



    fn http_status_code(&self) -> StatusCode {
        match self {
            // 委托给底层错误源
            BacktestStrategyError::BacktestNodeError { source, .. } => source.http_status_code(),
            BacktestStrategyError::NodeBenchmarkNotFount { source, .. } => source.http_status_code(),
            BacktestStrategyError::NodeCheckFailed { source, .. } => source.http_status_code(),
            BacktestStrategyError::NodeInitFailed { source, .. } => source.http_status_code(),
            BacktestStrategyError::NodeStopFailed { source, .. } => source.http_status_code(),

            // 服务器内部错误 (500)
            BacktestStrategyError::NodeInitTimeout { .. } |
            BacktestStrategyError::NodeStopTimeout { .. } |
            BacktestStrategyError::TokioTaskFailed { .. } |
            BacktestStrategyError::NodeStateNotReady { .. } |
            BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } |
            BacktestStrategyError::GetDataFailed { .. } |
            BacktestStrategyError::GetDataByDatetimeFailed { .. } |
            BacktestStrategyError::KlineDataLengthNotSame { .. } |
            BacktestStrategyError::PlayIndexOutOfRange { .. } |
            BacktestStrategyError::DivideByZero { .. } |
            BacktestStrategyError::NodeCycleDetected { .. } |
            BacktestStrategyError::StrategyStateInvalidTrans { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // 客户端错误 - 配置/数据问题 (400)
            BacktestStrategyError::NodeConfigNull { .. } |
            BacktestStrategyError::EdgeConfigNull { .. } |
            BacktestStrategyError::EdgeConfigMissField { .. } |
            BacktestStrategyError::GetStartNodeConfigFailed { .. } |
            BacktestStrategyError::IntervalNotSame { .. } |
            BacktestStrategyError::CusVarUpdateOpValueIsNone { .. } |
            BacktestStrategyError::UnSupportVariableOperation { .. } => StatusCode::BAD_REQUEST,

            // 客户端错误 - 资源未找到 (404)
            BacktestStrategyError::NodeNotFound { .. } |
            BacktestStrategyError::KlineKeyNotFound { .. } |
            BacktestStrategyError::CustomVariableNotExist { .. } |
            BacktestStrategyError::NodeBenchmarkNotFound { .. } => StatusCode::NOT_FOUND,

            // 客户端错误 - 冲突/状态错误 (409)
            BacktestStrategyError::AlreadyPlaying { .. } |
            BacktestStrategyError::AlreadyPausing { .. } => StatusCode::CONFLICT,

            // 成功但已完成 (200 - 虽然是错误但在业务上是正常完成)
            BacktestStrategyError::PlayFinished { .. } => StatusCode::OK,

            // 服务不可用 (503)
            BacktestStrategyError::UpdateStrategyStatusFailed { .. } => StatusCode::SERVICE_UNAVAILABLE,
        }
    }


    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // transparent errors - return source message directly
                BacktestStrategyError::BacktestNodeError { source, .. } => source.error_message(language),
                BacktestStrategyError::NodeBenchmarkNotFount { source, .. } => source.error_message(language),
                BacktestStrategyError::NodeCheckFailed { source, .. } => source.error_message(language),
                BacktestStrategyError::NodeInitFailed { source, .. } => source.error_message(language),
                BacktestStrategyError::NodeStopFailed { source, .. } => source.error_message(language),


                // non-transparent errors - use custom message
                BacktestStrategyError::NodeStopTimeout { node_name, .. } => {
                    format!("[{node_name}]停止超时")
                }
                BacktestStrategyError::NodeStateNotReady { node_name, .. } => {
                    format!("[{node_name}] 节点状态未就绪: 节点未准备好")
                }
                BacktestStrategyError::NodeInitTimeout { node_name, .. } => {
                    format!("[{node_name}] 节点初始化超时")
                }
                BacktestStrategyError::TokioTaskFailed { task_name, node_name, .. } => {
                    format!("执行 [{task_name}] 任务失败，节点: [{node_name}]")
                }
                BacktestStrategyError::NodeConfigNull {
                    ..
                } => {
                    format!("节点配置为空")
                }
                BacktestStrategyError::EdgeConfigNull {
                    strategy_name,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 边配置为空")
                }
                BacktestStrategyError::EdgeConfigMissField {
                    strategy_name,
                    field_name,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 边配置缺少字段: {field_name}")
                }
                BacktestStrategyError::NodeNotFound {
                    strategy_name,
                    node_id: node_name,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 节点 [{node_name}] 未找到")
                }
                BacktestStrategyError::StrategyStateInvalidTrans {
                    strategy_name,
                    current_state,
                    trans_trigger,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 无效的状态转换: 当前状态: {current_state}, 事件: {trans_trigger}")
                }
                BacktestStrategyError::UpdateStrategyStatusFailed {
                    strategy_name,
                    source,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 更新状态失败: {source}")
                }
                BacktestStrategyError::WaitAllNodesStoppedTimeout { .. } => {
                    format!("等待节点停止超时")
                }
                BacktestStrategyError::PlayFinished { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] 所有数据播放完毕")
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
                    format!("策略 [{strategy_name}] 获取数据失败: 数据键: {key}, 缓存索引: {play_index}")
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
                    format!("策略 [{strategy_name}] kline数据长度不相同")
                }
                BacktestStrategyError::KlineKeyNotFound { strategy_name, kline_key, .. } => {
                    format!("策略 [{strategy_name}] kline key 不存在: {kline_key}")
                }
                BacktestStrategyError::PlayIndexOutOfRange { strategy_name, kline_data_length, play_index, .. } => {
                    format!("策略 [{strategy_name}] 播放索引超出范围: k线数据长度: {kline_data_length}, 播放索引: {play_index}")
                }

                BacktestStrategyError::CustomVariableNotExist { var_name, .. } => {
                    format!("自定义变量[{var_name}]不存在.")
                }

                BacktestStrategyError::CusVarUpdateOpValueIsNone { var_name, .. } => {
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

                BacktestStrategyError::DivideByZero { var_name, .. } => {
                    format!("除零错误: 自定义变量[{var_name}]")
                }

                BacktestStrategyError::NodeBenchmarkNotFound { strategy_name, node_name, .. } => {
                    format!("策略 [{strategy_name}] 节点benchmark未找到: {node_name}")
                }

                BacktestStrategyError::NodeCycleDetected { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] 节点存在循环依赖")
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BacktestStrategyError::BacktestNodeError { source, .. } => generate_error_code_chain(source),
            BacktestStrategyError::NodeBenchmarkNotFount { source, .. } => generate_error_code_chain(source),
            // Errors with source - append self to source chain
            BacktestStrategyError::NodeCheckFailed { source, .. } => generate_error_code_chain(source),
            BacktestStrategyError::NodeInitFailed { source, .. } => generate_error_code_chain(source),

            _ => vec![self.error_code()],
        }
    }
}
