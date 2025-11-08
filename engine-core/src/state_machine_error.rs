use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait};
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum EngineStateMachineError {
    #[snafu(display("fail to transfer engine state, run_state: {run_state}, trans_trigger: {trans_trigger}"))]
    EngineTransition {
        run_state: String,
        trans_trigger: String,
        backtrace: Backtrace
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeStateMachineError
impl StarRiverErrorTrait for EngineStateMachineError {
    fn get_prefix(&self) -> &'static str {
        "ENGINE_STATE_MACHINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            EngineStateMachineError::EngineTransition { .. } => 1001,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                EngineStateMachineError::EngineTransition {
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
