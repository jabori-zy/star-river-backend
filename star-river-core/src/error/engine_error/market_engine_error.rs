// use super::super::exchange_client_error::ExchangeClientError;
// use super::exchange_engine_error::ExchangeEngineError;
use crate::error::ErrorCode;
use crate::error::error_trait::ErrorLanguage;
use crate::error::error_trait::StarRiverErrorTrait;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MarketEngineError {
    // #[snafu(transparent)]
    // ExchangeEngine { source: ExchangeEngineError, backtrace: Backtrace },

    #[snafu(transparent)]
    ExchangeClient { source: ExchangeClientError, backtrace: Backtrace },
}

impl StarRiverErrorTrait for MarketEngineError {
    fn get_prefix(&self) -> &'static str {
        "MARKET_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // MarketEngineError::ExchangeEngine { .. } => 1001, // 交易所引擎错误
            MarketEngineError::ExchangeClient { .. } => 1002, // 交易所客户端错误
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // MarketEngineError::ExchangeEngine { source, .. } => {
            //     let mut chain = source.error_code_chain();
            //     chain.push(self.error_code());
            //     chain
            // }

            MarketEngineError::ExchangeClient { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                // MarketEngineError::ExchangeEngine { source, .. } => {
                //     format!("交易所引擎错误: {}", source.error_message(language))
                // }
                MarketEngineError::ExchangeClient { source, .. } => {
                    format!("交易所客户端错误: {}", source.error_message(language))
                }
            },
        }
    }
}
