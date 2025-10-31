use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestNodeStateMachineError {
    #[snafu(display("fail to transfer node state, run_state: {run_state}, trans_trigger: {trans_trigger}"))]
    NodeTransition {
        run_state: String,
        trans_trigger: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeStateMachineError
impl crate::error::error_trait::StarRiverErrorTrait for BacktestNodeStateMachineError {
    fn get_prefix(&self) -> &'static str {
        "NODE_STATE_MACHINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            BacktestNodeStateMachineError::NodeTransition { .. } => 1001,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self, BacktestNodeStateMachineError::NodeTransition { .. })
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                BacktestNodeStateMachineError::NodeTransition {
                    run_state,
                    trans_trigger,
                    ..
                } => {
                    format!("状态转换失败，运行状态: {}, 触发事件: {}", run_state, trans_trigger)
                }
            },
        }
    }
}
