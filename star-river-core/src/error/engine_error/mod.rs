pub mod cache_engine_error;
// pub mod exchange_engine_error;
pub mod indicator_engine_error;
// pub mod market_engine_error;
// pub mod strategy_engine_error;
pub mod engine_state_machine_error;

pub use cache_engine_error::*;
// pub use exchange_engine_error::*;
pub use indicator_engine_error::*;
// pub use market_engine_error::*;
// pub use strategy_engine_error::*;
pub use engine_state_machine_error::*;


use snafu::{Backtrace, Snafu};
use crate::error::error_trait::StarRiverErrorTrait;
use crate::error::ErrorCode;
use std::collections::HashMap;
use crate::error::error_trait::ErrorLanguage;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]

pub enum EngineError {
    #[snafu(transparent)]
    CacheEngine { source: CacheEngineError, backtrace: Backtrace },

    // #[snafu(transparent)]
    // ExchangeEngine { source: ExchangeEngineError, backtrace: Backtrace },

    #[snafu(transparent)]
    IndicatorEngine { source: IndicatorEngineError, backtrace: Backtrace },

    // #[snafu(transparent)]
    // MarketEngine { source: MarketEngineError, backtrace: Backtrace },

    // #[snafu(transparent)]
    // StrategyEngine { source: StrategyEngineError, backtrace: Backtrace },

    #[snafu(transparent)]
    EngineStateMachine { source: EngineStateMachineError, backtrace: Backtrace },
}


impl StarRiverErrorTrait for EngineError {
    fn get_prefix(&self) -> &'static str {
        "ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            EngineError::CacheEngine { .. } => 1001, // 缓存引擎错误
            // EngineError::ExchangeEngine { .. } => 1002, // 交易所引擎错误
            EngineError::IndicatorEngine { .. } => 1003, // 指标引擎错误
            // EngineError::MarketEngine { .. } => 1004, // 市场引擎错误
            // EngineError::StrategyEngine { .. } => 1005, // 策略引擎错误
            EngineError::EngineStateMachine { .. } => 1006, // 引擎状态机错误
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            EngineError::CacheEngine { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }

            // EngineError::ExchangeEngine { source, .. } => {
            //     let mut chain = source.error_code_chain();
            //     chain.push(self.error_code());
            //     chain
            // }

            EngineError::IndicatorEngine { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }

            // EngineError::MarketEngine { source, .. } => {
            //     let mut chain = source.error_code_chain();
            //     chain.push(self.error_code());
            //     chain
            // }

            // EngineError::StrategyEngine { source, .. } => {
            //     let mut chain = source.error_code_chain();
            //     chain.push(self.error_code());
            //     chain
            // }

            EngineError::EngineStateMachine { source, .. } => {
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
                EngineError::CacheEngine { source, .. } => {
                    format!("缓存引擎错误: {}", source.error_message(language))
                }
                // EngineError::ExchangeEngine { source, .. } => {
                //     format!("交易所引擎错误: {}", source.error_message(language))
                // }
                EngineError::IndicatorEngine { source, .. } => {
                    format!("指标引擎错误: {}", source.error_message(language))
                }
                // EngineError::MarketEngine { source, .. } => {
                //     format!("市场引擎错误: {}", source.error_message(language))
                // }
                // EngineError::StrategyEngine { source, .. } => {
                //     format!("策略引擎错误: {}", source.error_message(language))
                // }
                EngineError::EngineStateMachine { source, .. } => {
                    format!("引擎状态机错误: {}", source.error_message(language))
                }
            },
        }
    }
}