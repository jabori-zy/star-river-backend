use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use super::super::node_error::BacktestStrategyNodeError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestStrategyError {

    #[snafu(display("{node_name}({node_id}) {node_type} node init error"))]
    NodeInit {
        node_id: String,
        node_name: String,
        node_type: String,
        source: BacktestStrategyNodeError,
        backtrace: Backtrace,
    },

    #[snafu(display("{node_name}({node_id}) {node_type} node not ready"))]
    NodeStateNotReady {
        node_id: String,
        node_name: String,
        node_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display("{node_name}({node_id}) {node_type} node init timeout"))]
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
                BacktestStrategyError::NodeInit { .. } => 1001,
                BacktestStrategyError::NodeInitTimeout { .. } => 1002,
                BacktestStrategyError::TokioTaskFailed { .. } => 1003,
                BacktestStrategyError::NodeStateNotReady { .. } => 1004,
            };   

            format!("{}_{:04}", prefix, code)
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
            BacktestStrategyError::NodeStateNotReady { .. }
        )
    }
}