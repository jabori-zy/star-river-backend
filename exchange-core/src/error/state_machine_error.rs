use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeStateMachineError {
    #[snafu(display("fail to transfer node state, run_state: {run_state}, trans_trigger: {trans_trigger}"))]
    ExchangeTransition {
        run_state: String,
        trans_trigger: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for BacktestNodeStateMachineError
impl StarRiverErrorTrait for ExchangeStateMachineError {
    fn get_prefix(&self) -> &'static str {
        "EXCHANGE_STATE_MACHINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            ExchangeStateMachineError::ExchangeTransition { .. } => 1001,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ExchangeStateMachineError::ExchangeTransition {
                    run_state, trans_trigger, ..
                } => {
                    format!("状态转换失败，运行状态: {}, 触发事件: {}", run_state, trans_trigger)
                }
            },
        }
    }
}
