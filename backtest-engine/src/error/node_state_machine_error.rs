use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage, StatusCode};

use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestNodeStateMachineError {
    #[snafu(display("fail to transfer node state, run_state: {run_state}, trans_trigger: {trans_trigger}"))]
    NodeTransFailed {
        run_state: String,
        trans_trigger: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeStateMachineError
impl StarRiverErrorTrait for BacktestNodeStateMachineError {
    fn get_prefix(&self) -> &'static str {
        "NODE_STATE_MACHINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            BacktestNodeStateMachineError::NodeTransFailed { .. } => 1001,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            BacktestNodeStateMachineError::NodeTransFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BacktestNodeStateMachineError::NodeTransFailed {
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
