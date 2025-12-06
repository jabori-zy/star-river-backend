use std::{error::Error, sync::Arc};

use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode, generate_error_code_chain},
};

use super::node_error::NodeError;
use crate::benchmark::strategy_benchmark::NodeBenchmarkNotFountError;
// use event_center::EventCenterError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StrategyError {
    #[snafu(transparent)]
    StrategyBenchmarkError {
        source: NodeBenchmarkNotFountError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node check failed: {source}"))]
    NodeCheckFailed {
        strategy_name: String,
        node_name: String,
        source: NodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node init failed: {source}"))]
    NodeInitFailed {
        strategy_name: String,
        node_name: String,
        source: NodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] {node_name} node stop failed: {source}"))]
    NodeStopFailed {
        strategy_name: String,
        node_name: String,
        source: NodeError,
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
    NodeConfigNull { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] edges config is null"))]
    EdgeConfigNull { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] edges config miss field: {field_name}"))]
    EdgeConfigMissField {
        strategy_name: String,
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] node [{node_id}] not found"))]
    NodeNotFoundById {
        strategy_name: String,
        node_id: NodeId,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] node [{node_index}] not found"))]
    NodeNotFoundByIndex {
        strategy_name: String,
        node_index: usize,
        backtrace: Backtrace,
    },

    #[snafu(display("wait for all nodes to stop timeout"))]
    WaitAllNodesStoppedTimeout { backtrace: Backtrace },

    #[snafu(display("custom variable [{var_name}] not exists"))]
    CustomVariableNotExist { var_name: String, backtrace: Backtrace },

    #[snafu(display("the update operation value of custom variable [{var_name}] is none "))]
    CusVarUpdateOpValueIsNone { var_name: String, backtrace: Backtrace },

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
    DivideByZero { var_name: String, backtrace: Backtrace },

    #[snafu(display("[{strategy_name}] node benchmark not found: {node_name}"))]
    NodeBenchmarkNotFound {
        strategy_name: String,
        node_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("[{strategy_name}] node cycle detected"))]
    NodeCycleDetected { strategy_name: String, backtrace: Backtrace },

    #[snafu(display("#[{strategy_name}] node @[{node_name}] command send failed, reason: {source}"))]
    NodeCmdSendFailed {
        strategy_name: String,
        node_name: String,
        source: Arc<dyn Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },

    #[snafu(display("#[{strategy_name}] node @[{node_name}] command response receive failed, reason: {source}"))]
    NodeCmdRespRecvFailed {
        strategy_name: String,
        node_name: String,
        source: tokio::sync::oneshot::error::RecvError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for Mt5Error
impl StarRiverErrorTrait for StrategyError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_STRATEGY"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StrategyError::StrategyBenchmarkError { .. } => 1001,     // Node benchmark not found
            StrategyError::NodeCheckFailed { .. } => 1002,            // Node check error
            StrategyError::NodeInitFailed { .. } => 1003,             // Node initialization error
            StrategyError::NodeStopFailed { .. } => 1004,             // Node stop error
            StrategyError::NodeInitTimeout { .. } => 1005,            // Node initialization timeout
            StrategyError::NodeStopTimeout { .. } => 1006,            // Node stop timeout
            StrategyError::TokioTaskFailed { .. } => 1007,            // Task execution failed
            StrategyError::NodeStateNotReady { .. } => 1008,          // Node state not ready
            StrategyError::NodeConfigNull { .. } => 1009,             // Node configuration is null
            StrategyError::EdgeConfigNull { .. } => 1010,             // Edge configuration is null
            StrategyError::EdgeConfigMissField { .. } => 1011,        // Edge configuration missing field
            StrategyError::NodeNotFoundById { .. } => 1012,           // Node not found
            StrategyError::NodeNotFoundByIndex { .. } => 1013,        // Node not found (by index)
            StrategyError::WaitAllNodesStoppedTimeout { .. } => 1014, // Wait for all nodes to stop timeout
            StrategyError::CustomVariableNotExist { .. } => 1015,     // Custom variable does not exist
            StrategyError::CusVarUpdateOpValueIsNone { .. } => 1016,  // Custom variable update operation value is none
            StrategyError::UnSupportVariableOperation { .. } => 1017, // Unsupported variable operation
            StrategyError::DivideByZero { .. } => 1018,               // Divide by zero error
            StrategyError::NodeBenchmarkNotFound { .. } => 1019,      // Node benchmark not found
            StrategyError::NodeCycleDetected { .. } => 1020,          // Node circular dependency detected
            StrategyError::NodeCmdSendFailed { .. } => 1021,          // Node command send failed
            StrategyError::NodeCmdRespRecvFailed { .. } => 1022,      // Node command response receive failed
        };
        format!("{prefix}_{code}")
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            // Delegate to underlying error source
            StrategyError::StrategyBenchmarkError { source, .. } => source.http_status_code(),
            StrategyError::NodeCheckFailed { source, .. } => source.http_status_code(),
            StrategyError::NodeInitFailed { source, .. } => source.http_status_code(),
            StrategyError::NodeStopFailed { source, .. } => source.http_status_code(),

            // Server internal error (500)
            StrategyError::NodeInitTimeout { .. }
            | StrategyError::NodeStopTimeout { .. }
            | StrategyError::TokioTaskFailed { .. }
            | StrategyError::NodeStateNotReady { .. }
            | StrategyError::WaitAllNodesStoppedTimeout { .. }
            | StrategyError::DivideByZero { .. }
            | StrategyError::NodeCycleDetected { .. }
            | StrategyError::NodeCmdSendFailed { .. }
            | StrategyError::NodeCmdRespRecvFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // Client error - configuration/data issues (400)
            StrategyError::NodeConfigNull { .. }
            | StrategyError::EdgeConfigNull { .. }
            | StrategyError::EdgeConfigMissField { .. }
            | StrategyError::CusVarUpdateOpValueIsNone { .. }
            | StrategyError::UnSupportVariableOperation { .. } => StatusCode::BAD_REQUEST,

            // Client error - resource not found (404)
            StrategyError::NodeNotFoundById { .. }
            | StrategyError::NodeNotFoundByIndex { .. }
            | StrategyError::CustomVariableNotExist { .. }
            | StrategyError::NodeBenchmarkNotFound { .. } => StatusCode::NOT_FOUND,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // transparent errors - return source message directly
                StrategyError::StrategyBenchmarkError { source, .. } => source.error_message(language),
                StrategyError::NodeCheckFailed { source, .. } => source.error_message(language),
                StrategyError::NodeInitFailed { source, .. } => source.error_message(language),
                StrategyError::NodeStopFailed { source, .. } => source.error_message(language),

                // non-transparent errors - use custom message
                StrategyError::NodeStopTimeout { node_name, .. } => {
                    format!("[{node_name}]停止超时")
                }
                StrategyError::NodeStateNotReady { node_name, .. } => {
                    format!("[{node_name}] 节点状态未就绪: 节点未准备好")
                }
                StrategyError::NodeInitTimeout { node_name, .. } => {
                    format!("[{node_name}] 节点初始化超时")
                }
                StrategyError::TokioTaskFailed { task_name, node_name, .. } => {
                    format!("执行 [{task_name}] 任务失败，节点: [{node_name}]")
                }
                StrategyError::NodeConfigNull { .. } => {
                    format!("节点配置为空")
                }
                StrategyError::EdgeConfigNull { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] 边配置为空")
                }
                StrategyError::EdgeConfigMissField {
                    strategy_name, field_name, ..
                } => {
                    format!("策略 [{strategy_name}] 边配置缺少字段: {field_name}")
                }
                StrategyError::NodeNotFoundById {
                    strategy_name,
                    node_id: node_name,
                    ..
                } => {
                    format!("策略 [{strategy_name}] 节点 [{node_name}] 未找到")
                }
                StrategyError::WaitAllNodesStoppedTimeout { .. } => {
                    format!("等待节点停止超时")
                }

                StrategyError::NodeNotFoundByIndex {
                    strategy_name, node_index, ..
                } => {
                    format!("策略 [{strategy_name}] 的索引为 [{node_index}] 的节点未找到")
                }

                StrategyError::CustomVariableNotExist { var_name, .. } => {
                    format!("自定义变量[{var_name}]不存在.")
                }

                StrategyError::CusVarUpdateOpValueIsNone { var_name, .. } => {
                    format!("变量[{var_name}]的更新操作值为空")
                }
                StrategyError::UnSupportVariableOperation {
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

                StrategyError::DivideByZero { var_name, .. } => {
                    format!("除零错误: 自定义变量[{var_name}]")
                }

                StrategyError::NodeBenchmarkNotFound {
                    strategy_name, node_name, ..
                } => {
                    format!("策略 [{strategy_name}] 节点benchmark未找到: {node_name}")
                }

                StrategyError::NodeCycleDetected { strategy_name, .. } => {
                    format!("策略 [{strategy_name}] 节点存在循环依赖")
                }
                StrategyError::NodeCmdSendFailed {
                    strategy_name,
                    node_name,
                    source,
                    ..
                } => {
                    format!("#[{strategy_name}] node @[{node_name}] command send failed, reason: {source}")
                }
                StrategyError::NodeCmdRespRecvFailed {
                    strategy_name,
                    node_name,
                    source,
                    ..
                } => {
                    format!("#[{strategy_name}] node @[{node_name}] command response receive failed, reason: {source}")
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            StrategyError::StrategyBenchmarkError { source, .. } => generate_error_code_chain(source, self.error_code()),
            // Errors with source - append self to source chain
            StrategyError::NodeCheckFailed { source, .. } => generate_error_code_chain(source, self.error_code()),
            StrategyError::NodeInitFailed { source, .. } => generate_error_code_chain(source, self.error_code()),

            _ => vec![self.error_code()],
        }
    }
}
