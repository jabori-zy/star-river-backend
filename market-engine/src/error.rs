
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use snafu::{Backtrace, Snafu};
use exchange_engine::error::ExchangeEngineError;
use engine_core::state_machine_error::EngineStateMachineError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MarketEngineError {
    // #[snafu(transparent)]
    // ExchangeEngine { source: ExchangeEngineError, backtrace: Backtrace },

    #[snafu(transparent)]
    ExchangeEngineError { source: ExchangeEngineError, backtrace: Backtrace },

    #[snafu(transparent)]
    StateMachineError { source: EngineStateMachineError, backtrace: Backtrace },
}

impl StarRiverErrorTrait for MarketEngineError {
    fn get_prefix(&self) -> &'static str {
        "MARKET_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            MarketEngineError::ExchangeEngineError { .. } => 1001, // 交易所引擎错误
            MarketEngineError::StateMachineError { .. } => 1002, // 状态机错误
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            MarketEngineError::ExchangeEngineError { source, .. } => generate_error_code_chain(source),
            MarketEngineError::StateMachineError { source, .. } => generate_error_code_chain(source)
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                MarketEngineError::ExchangeEngineError { source, .. } => source.error_message(language),
                MarketEngineError::StateMachineError { source, .. } => source.error_message(language)
            },
        }
    }
}
