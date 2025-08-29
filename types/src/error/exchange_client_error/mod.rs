pub mod mt5_error;
pub mod data_processor_error;

pub use mt5_error::*;
pub use data_processor_error::*;
use snafu::{Snafu, Backtrace};
use std::collections::HashMap;

use crate::error::ErrorCode;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeClientError {
    #[snafu(transparent)]
    MetaTrader5 {
        source: Mt5Error,
        backtrace: Backtrace,
    },
    
    #[snafu(transparent)]
    DataProcessor {
        source: DataProcessorError,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Binance error: {message}"))]
    Binance {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Authentication error: {message}"))]
    Authentication {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Rate limit exceeded: {message}"))]
    RateLimit {
        message: String,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Internal error: {message}"))]
    Internal {
        message: String,
        backtrace: Backtrace,
    },
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
                    ExchangeClientError::Authentication { .. } => 1006,
                    ExchangeClientError::Internal { .. } => 1007,
                    _ => unreachable!(),
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
            _ => matches!(self,
                // Network-related errors are usually recoverable
                ExchangeClientError::RateLimit { .. } |
                
                // Binance-specific errors may be recoverable
                ExchangeClientError::Binance { .. }
            )
        }
    }
}