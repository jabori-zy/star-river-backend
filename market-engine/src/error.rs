use engine_core::state_machine_error::EngineStateMachineError;
use exchange_engine::error::ExchangeEngineError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::AccountId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
    exchange::Exchange,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MarketEngineError {
    #[snafu(transparent)]
    ExchangeEngineError { source: ExchangeEngineError, backtrace: Backtrace },
    #[snafu(transparent)]
    StateMachineError {
        source: EngineStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("account {account_id}'s exchange {exchange} is not registered"))]
    ExchangeNotRegistered {
        account_id: AccountId,
        exchange: Exchange,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for MarketEngineError {
    fn get_prefix(&self) -> &'static str {
        "MARKET_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            MarketEngineError::ExchangeEngineError { .. } => 1001,   // Exchange engine error
            MarketEngineError::StateMachineError { .. } => 1002,     // State machine error
            MarketEngineError::ExchangeNotRegistered { .. } => 1003, // Exchange not registered
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            MarketEngineError::ExchangeEngineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            MarketEngineError::StateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            MarketEngineError::ExchangeNotRegistered { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                MarketEngineError::ExchangeEngineError { source, .. } => source.error_message(language),
                MarketEngineError::StateMachineError { source, .. } => source.error_message(language),
                MarketEngineError::ExchangeNotRegistered { account_id, exchange, .. } => {
                    format!("账户 {account_id} 交易所 {exchange} 未注册")
                }
            },
        }
    }
}
