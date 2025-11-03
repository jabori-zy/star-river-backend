pub mod binance_error;
pub mod data_processor_error;
pub mod mt5_error;
pub mod exchange_state_machine_error;

use crate::error::ErrorCode;
use crate::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};
use binance_error::BinanceError;
pub use data_processor_error::*;
use mt5_error::Mt5Error;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use exchange_state_machine_error::ExchangeStateMachineError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeClientError {
    #[snafu(transparent)]
    MetaTrader5 { source: Mt5Error, backtrace: Backtrace },

    #[snafu(transparent)]
    Binance { source: BinanceError, backtrace: Backtrace },

    #[snafu(transparent)]
    ExchangeStateMachine { source: ExchangeStateMachineError, backtrace: Backtrace },

    #[snafu(transparent)]
    DataProcessor { source: DataProcessorError, backtrace: Backtrace },

    #[snafu(display("Authentication error: {message}"))]
    Authentication { message: String, backtrace: Backtrace },

    #[snafu(display("Rate limit exceeded: {message}"))]
    RateLimit { message: String, backtrace: Backtrace },

    #[snafu(display("Internal error: {message}"))]
    Internal { message: String, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for ExchangeClientError
impl crate::error::error_trait::StarRiverErrorTrait for ExchangeClientError {
    fn get_prefix(&self) -> &'static str {
        match self {
            // For nested errors, delegate to the inner error's prefix
            ExchangeClientError::MetaTrader5 { source, .. } => source.get_prefix(),
            ExchangeClientError::DataProcessor { source, .. } => source.get_prefix(),
            _ => "EXCHANGE_CLIENT",
        }
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            // For nested errors, delegate to the inner error's code
            ExchangeClientError::MetaTrader5 { source, .. } => source.error_code(),
            ExchangeClientError::DataProcessor { source, .. } => source.error_code(),

            // For direct exchange client errors, use EXCHANGE_CLIENT prefix
            _ => {
                let prefix = "EXCHANGE_CLIENT";
                let code = match self {
                    ExchangeClientError::Binance { .. } => 1001,
                    ExchangeClientError::MetaTrader5 { .. } => 1002,
                    ExchangeClientError::DataProcessor { .. } => 1003,
                    ExchangeClientError::ExchangeStateMachine { .. } => 1004,
                    ExchangeClientError::Authentication { .. } => 1005,
                    ExchangeClientError::RateLimit { .. } => 1006,
                    ExchangeClientError::Internal { .. } => 1007,
                };
                format!("{}_{:04}", prefix, code)
            }
        }
    }

    fn context(&self) -> HashMap<&'static str, String> {
        match self {
            ExchangeClientError::MetaTrader5 { source, .. } => source.context(),
            _ => HashMap::new(),
        }
    }

    fn is_recoverable(&self) -> bool {
        match self {
            // For nested errors, delegate to the inner error's recoverability
            ExchangeClientError::MetaTrader5 { source, .. } => source.is_recoverable(),
            ExchangeClientError::DataProcessor { source, .. } => source.is_recoverable(),

            // Recoverable errors (network, connection, temporary issues, trading operations)
            _ => matches!(
                self,
                // Network-related errors are usually recoverable
                ExchangeClientError::RateLimit { .. } |
                // Binance-specific errors may be recoverable
                ExchangeClientError::Binance { .. }
            ),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ExchangeClientError::MetaTrader5 { source, .. } => {
                    format!("MetaTrader5错误: {}", source.error_message(language))
                }
                ExchangeClientError::ExchangeStateMachine { source, .. } => {
                    format!("交易所状态机错误: {}", source.error_message(language))
                }
                ExchangeClientError::DataProcessor { source, .. } => {
                    format!("数据处理器错误: {}", source.error_message(language))
                }
                ExchangeClientError::Binance { source, .. } => {
                    format!("币安错误: {}", source.error_message(language))
                }
                ExchangeClientError::Authentication { message, .. } => {
                    format!("认证错误: {}", message)
                }
                ExchangeClientError::RateLimit { message, .. } => {
                    format!("频率限制超过: {}", message)
                }
                ExchangeClientError::Internal { message, .. } => {
                    format!("内部错误: {}", message)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // transparent errors - delegate to source
            ExchangeClientError::MetaTrader5 { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            ExchangeClientError::Binance { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            ExchangeClientError::DataProcessor { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }

            // non-transparent errors - return own error code
            _ => vec![self.error_code()],
        }
    }
}
