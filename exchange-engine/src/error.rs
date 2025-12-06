//workspace crate
use database::error::DatabaseError;
use engine_core::state_machine_error::EngineStateMachineError;
use exchange_client::{binance::error::BinanceError, metatrader5::error::Mt5Error};
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::AccountId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
    exchange::Exchange,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeEngineError {
    #[snafu(transparent)]
    EngineStateMachineError {
        source: EngineStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    BinanceError { source: BinanceError, backtrace: Backtrace },

    #[snafu(transparent)]
    Mt5Error { source: Mt5Error, backtrace: Backtrace },

    #[snafu(transparent)]
    DatabaseError { source: DatabaseError, backtrace: Backtrace },

    #[snafu(display("account {account_id}'s exchange type {:?} is unsupported", exchange_type))]
    UnsupportedExchangeType {
        exchange_type: Exchange,
        account_id: AccountId,
        backtrace: Backtrace,
    },

    // === Exchange Client Management Errors ===
    #[snafu(display("exchange {exchange_name} is not registered. account id is {account_id}"))]
    ExchangeClientNotRegistered {
        account_id: AccountId,
        exchange_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Binance register failed for {exchange_name}"))]
    BinanceRegisterFailed {
        exchange_name: String,
        #[snafu(source)]
        source: BinanceError,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for ExchangeEngineError
impl StarRiverErrorTrait for ExchangeEngineError {
    fn get_prefix(&self) -> &'static str {
        "EXCHANGE_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeEngineError::Mt5Error { source, .. } => source.error_code(),
            ExchangeEngineError::BinanceError { source, .. } => source.error_code(),
            ExchangeEngineError::EngineStateMachineError { source, .. } => source.error_code(),

            // For direct exchange engine errors, use EXCHANGE_ENGINE prefix
            _ => {
                let prefix = "EXCHANGE_ENGINE";
                let code = match self {
                    // Registration & Configuration (1002-1004)
                    ExchangeEngineError::BinanceError { .. } => 1002,            // Binance error
                    ExchangeEngineError::Mt5Error { .. } => 1003,                // MetaTrader5 error
                    ExchangeEngineError::EngineStateMachineError { .. } => 1004, // Engine state machine error
                    ExchangeEngineError::DatabaseError { .. } => 1005,           // Database error
                    ExchangeEngineError::UnsupportedExchangeType { .. } => 1006, // Unsupported exchange type

                    // Exchange Client Management (1011-1013)
                    ExchangeEngineError::ExchangeClientNotRegistered { .. } => 1007, // Exchange client not found
                    ExchangeEngineError::BinanceRegisterFailed { .. } => 1008,       // Binance registration failed
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            ExchangeEngineError::BinanceError { source, .. } => generate_error_code_chain(source, self.error_code()),
            ExchangeEngineError::Mt5Error { source, .. } => generate_error_code_chain(source, self.error_code()),
            ExchangeEngineError::EngineStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            ExchangeEngineError::DatabaseError { source, .. } => generate_error_code_chain(source, self.error_code()),
            ExchangeEngineError::BinanceRegisterFailed { source, .. } => generate_error_code_chain(source, self.error_code()),

            // For errors without source or with external sources
            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ExchangeEngineError::BinanceError { source, .. } => source.error_message(language),
                ExchangeEngineError::Mt5Error { source, .. } => source.error_message(language),
                ExchangeEngineError::EngineStateMachineError { source, .. } => source.error_message(language),
                ExchangeEngineError::DatabaseError { source, .. } => {
                    format!("数据库错误: {}", source.error_message(language))
                }
                ExchangeEngineError::UnsupportedExchangeType {
                    exchange_type, account_id, ..
                } => {
                    format!("账户 {} 的交易所类型 {:?} 不支持", account_id, exchange_type)
                }
                ExchangeEngineError::ExchangeClientNotRegistered {
                    exchange_name, account_id, ..
                } => {
                    format!("客户端 {} 未注册。 客户端id: {}", exchange_name, account_id)
                }
                ExchangeEngineError::BinanceRegisterFailed { exchange_name, source, .. } => {
                    format!(
                        "币安注册失败: 交易所名称: {}, 原因: {}",
                        exchange_name,
                        source.error_message(language)
                    )
                }
            },
        }
    }
}
