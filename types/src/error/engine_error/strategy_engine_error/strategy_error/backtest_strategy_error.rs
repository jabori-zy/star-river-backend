use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use super::super::node_error::BacktestStrategyNodeError;
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

    #[snafu(display("backtest strategy node state not ready: {node_name}({node_id}) {node_type} node not ready"))]
    NodeStateNotReady {
        node_id: String,
        node_name: String,
        node_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display("backtest strategy node init timeout: {node_name}({node_id}) {node_type} node init timeout"))]
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

    #[snafu(display("NODE_CONFIG_NULL: strategy [{strategy_name}({strategy_id})] nodes config is null"))]
    NodeConfigNull {
        strategy_id: i32,
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("EDGE_CONFIG_NULL: strategy [{strategy_name}({strategy_id})] edges config is null"))]
    EdgeConfigNull {
        strategy_id: i32,
        strategy_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("EDGE_CONFIG_MISS_FIELD: strategy [{strategy_name}({strategy_id})] edges config miss field: {field_name}"))]
    EdgeConfigMissField {
        strategy_id: i32,
        strategy_name: String,
        field_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("NODE_NOT_FOUND: strategy [{strategy_name}({strategy_id})] node not found"))]
    NodeNotFound {
        strategy_id: i32,
        strategy_name: String,
        node_id: String,
        backtrace: Backtrace,
    },

    #[snafu(display("STRATEGY_STATE_INVALID_STATE_TRANSITION: strategy [{strategy_name}({strategy_id})] invalid state transition: current state: {current_state}, event: {event}"))]
    StrategyStateInvalidStateTransition {
        strategy_id: i32,
        strategy_name: String,
        current_state: String,
        event: String,
        backtrace: Backtrace,
    },

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
                BacktestStrategyError::NodeCheck { .. } => 1000,
                BacktestStrategyError::NodeInit { .. } => 1001,
                BacktestStrategyError::NodeInitTimeout { .. } => 1002,
                BacktestStrategyError::TokioTaskFailed { .. } => 1003,
                BacktestStrategyError::NodeStateNotReady { .. } => 1004,
                BacktestStrategyError::NodeConfigNull { .. } => 1005,
                BacktestStrategyError::EdgeConfigNull { .. } => 1006,
                BacktestStrategyError::EdgeConfigMissField { .. } => 1007,
                BacktestStrategyError::NodeNotFound { .. } => 1008,
                BacktestStrategyError::StrategyStateInvalidStateTransition { .. } => 1009,
                // BacktestStrategyError::EventSendError { .. } => 1010,
            };
            format!("{prefix}_{code}")
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx

           
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            BacktestStrategyError::NodeInit { .. } |
            BacktestStrategyError::NodeInitTimeout { .. } |
            BacktestStrategyError::TokioTaskFailed { .. } |
            BacktestStrategyError::NodeStateNotReady { .. } |
            BacktestStrategyError::NodeConfigNull { .. } |
            BacktestStrategyError::EdgeConfigNull { .. } |
            BacktestStrategyError::EdgeConfigMissField { .. } |
            BacktestStrategyError::NodeNotFound { .. } |
            BacktestStrategyError::StrategyStateInvalidStateTransition { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => {
                self.to_string()
            },
            Language::Chinese => {
                match self {
                    BacktestStrategyError::NodeCheck { source, .. } => {
                        format!("回测策略节点检查错误: {}", source.get_error_message(language))
                    },
                    BacktestStrategyError::NodeInit { source, .. } => {
                        format!("回测策略节点初始化错误: {}", source.get_error_message(language))
                    },
                    BacktestStrategyError::NodeStateNotReady { node_name, node_id, node_type, .. } => {
                        format!("回测策略节点状态未就绪: {}({}) {} 节点未准备好", node_name, node_id, node_type)
                    },
                    BacktestStrategyError::NodeInitTimeout { node_name, node_id, node_type, .. } => {
                        format!("回测策略节点初始化超时: {}({}) {} 节点初始化超时", node_name, node_id, node_type)
                    },
                    BacktestStrategyError::TokioTaskFailed { task_name, node_name, node_id, node_type, .. } => {
                        format!("执行 {} 任务失败，节点: {}({}) {}", task_name, node_name, node_id, node_type)
                    },
                    BacktestStrategyError::NodeConfigNull { strategy_name, strategy_id, .. } => {
                        format!("节点配置为空: 策略 [{}({})] 节点配置为空", strategy_name, strategy_id)
                    },
                    BacktestStrategyError::EdgeConfigNull { strategy_name, strategy_id, .. } => {
                        format!("边配置为空: 策略 [{}({})] 边配置为空", strategy_name, strategy_id)
                    },
                    BacktestStrategyError::EdgeConfigMissField { strategy_name, strategy_id, field_name, .. } => {
                        format!("边配置缺少字段: 策略 [{}({})] 边配置缺少字段: {}", strategy_name, strategy_id, field_name)
                    },
                    BacktestStrategyError::NodeNotFound { strategy_name, strategy_id, node_id, .. } => {
                        format!("节点未找到: 策略 [{}({})] 节点 {} 未找到", strategy_name, strategy_id, node_id)
                    },
                    BacktestStrategyError::StrategyStateInvalidStateTransition { strategy_name, strategy_id, current_state, event, .. } => {
                        format!("策略状态转换无效: 策略 [{}({})] 无效的状态转换: 当前状态: {}, 事件: {}", strategy_name, strategy_id, current_state, event)
                    },
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // Errors with source - append self to source chain
            BacktestStrategyError::NodeCheck { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            BacktestStrategyError::NodeInit { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            },
            BacktestStrategyError::NodeInitTimeout { .. } => {
                // tokio::time::error::Elapsed doesn't implement our trait, so we start the chain here
                vec![self.error_code()]
            },
            BacktestStrategyError::TokioTaskFailed { .. } => {
                // tokio::task::JoinError doesn't implement our trait, so we start the chain here
                vec![self.error_code()]
            },
            
            // Errors without source - use default implementation
            _ => vec![self.error_code()],
        }
    }
}