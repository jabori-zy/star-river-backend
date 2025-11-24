use engine_core::state_machine_error::EngineStateMachineError;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use ta_lib::error::TaLibError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum IndicatorEngineError {
    #[snafu(transparent)]
    TalibError { source: TaLibError, backtrace: Backtrace },

    #[snafu(transparent)]
    StateMachineError {
        source: EngineStateMachineError,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for IndicatorEngineError {
    fn get_prefix(&self) -> &'static str {
        "INDICATOR_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            IndicatorEngineError::TalibError { .. } => 1001,        // TA-Lib错误
            IndicatorEngineError::StateMachineError { .. } => 1002, // 状态机错误
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            IndicatorEngineError::TalibError { .. } => vec!["TA_LIB_ERROR".to_string()],
            IndicatorEngineError::StateMachineError { source, .. } => generate_error_code_chain(source),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                IndicatorEngineError::TalibError { source, .. } => format!("TA-Lib错误: {}", source),
                IndicatorEngineError::StateMachineError { source, .. } => source.error_message(language),
            },
        }
    }
}
