pub mod node_error;
pub mod strategy_error;


use snafu::{Snafu, Backtrace};
use std::collections::HashMap;
use crate::error::ErrorCode;
use strategy_error::BacktestStrategyError;


#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StrategyEngineError {
    #[snafu(transparent)]
    BacktestStrategyError {
        source: BacktestStrategyError,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy type {} is unsupported", strategy_type))]
    UnsupportedStrategyType {
        strategy_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display("strategy {} is exists", strategy_id))]
    StrategyIsExist {
        strategy_id: i32,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for StrategyEngineError
impl crate::error::error_trait::StarRiverErrorTrait for StrategyEngineError {
    fn get_prefix(&self) -> &'static str {
        "STRATEGY_ENGINE"
    }
    
    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // For nested errors, delegate to the inner error's code
            StrategyEngineError::BacktestStrategyError { .. } => 1001,
            StrategyEngineError::UnsupportedStrategyType { .. } => 1002,
            StrategyEngineError::StrategyIsExist { .. } => 1003,
        };
        format!("{}_{:04}", prefix, code)
    }
    
    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self,
            StrategyEngineError::BacktestStrategyError { .. } |
            StrategyEngineError::UnsupportedStrategyType { .. } |
            StrategyEngineError::StrategyIsExist { .. }
        )
    }
    
}






