use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};

use crate::{binance::error::BinanceError, metatrader5::error::Mt5Error};

/// Unified error type for all exchange clients
///
/// This enum wraps exchange-specific errors and provides a unified interface
/// for error handling across different exchange implementations.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ExchangeError {
    /// Binance exchange error
    #[snafu(transparent)]
    Binance { source: BinanceError, backtrace: Backtrace },

    /// MetaTrader5 exchange error
    #[snafu(transparent)]
    Mt5 { source: Mt5Error, backtrace: Backtrace },
}

impl StarRiverErrorTrait for ExchangeError {
    fn get_prefix(&self) -> &'static str {
        "EXCHANGE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            ExchangeError::Binance { .. } => 1001,
            ExchangeError::Mt5 { .. } => 1002,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            ExchangeError::Binance { source, .. } => generate_error_code_chain(source),
            ExchangeError::Mt5 { source, .. } => generate_error_code_chain(source),
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match self {
            ExchangeError::Binance { source, .. } => source.error_message(language),
            ExchangeError::Mt5 { source, .. } => source.error_message(language),
        }
    }
}
