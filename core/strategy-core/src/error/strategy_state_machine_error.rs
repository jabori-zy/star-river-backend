use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StrategyStateMachineError {
    #[snafu(display("[{strategy_name}] fail to transfer strategy state, run_state: {run_state}, trans_trigger: {trans_trigger}"))]
    StrategyStateTransFailed {
        strategy_name: String,
        run_state: String,
        trans_trigger: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeStateMachineError
impl StarRiverErrorTrait for StrategyStateMachineError {
    fn get_prefix(&self) -> &'static str {
        "STRATEGY_STATE_MACHINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StrategyStateMachineError::StrategyStateTransFailed { .. } => 1001,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            StrategyStateMachineError::StrategyStateTransFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                StrategyStateMachineError::StrategyStateTransFailed {
                    strategy_name,
                    run_state,
                    trans_trigger,
                    ..
                } => {
                    format!(
                        "[{strategy_name}] 策略状态转换失败，运行状态: {}, 触发事件: {}",
                        run_state, trans_trigger
                    )
                }
            },
        }
    }
}
