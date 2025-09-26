pub mod node_error;
pub mod strategy_error;

use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
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
    UnsupportedStrategyType { strategy_type: String, backtrace: Backtrace },

    #[snafu(display("strategy {} is exists", strategy_id))]
    StrategyIsExist { strategy_id: i32, backtrace: Backtrace },

    #[snafu(display("strategy instance not found: {}", strategy_id))]
    StrategyInstanceNotFound { strategy_id: i32, backtrace: Backtrace },

    #[snafu(display("database error: {}", source))]
    Database { source: DbErr, backtrace: Backtrace },

    #[snafu(display("trade mode {} is unsupported", trade_mode))]
    UnsupportedTradeMode { trade_mode: String, backtrace: Backtrace },

    #[snafu(display("strategy {strategy_id} config not found"))]
    StrategyConfigNotFound {
        strategy_id: i32,
        source: DbErr,
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
            StrategyEngineError::StrategyInstanceNotFound { .. } => 1004,
            StrategyEngineError::Database { .. } => 1005,
            StrategyEngineError::UnsupportedTradeMode { .. } => 1006,
            StrategyEngineError::StrategyConfigNotFound { .. } => 1007,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            StrategyEngineError::BacktestStrategyError { .. }
                | StrategyEngineError::UnsupportedStrategyType { .. }
                | StrategyEngineError::StrategyIsExist { .. }
                | StrategyEngineError::StrategyInstanceNotFound { .. }
                | StrategyEngineError::Database { .. }
                | StrategyEngineError::UnsupportedTradeMode { .. }
                | StrategyEngineError::StrategyConfigNotFound { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // For transparent errors, delegate to the inner error's chain
            StrategyEngineError::BacktestStrategyError { source, .. } => source.error_code_chain(),

            // For errors without source
            StrategyEngineError::UnsupportedStrategyType { .. }
            | StrategyEngineError::StrategyIsExist { .. }
            | StrategyEngineError::StrategyInstanceNotFound { .. }
            | StrategyEngineError::Database { .. }
            | StrategyEngineError::UnsupportedTradeMode { .. }
            | StrategyEngineError::StrategyConfigNotFound { .. } => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                StrategyEngineError::BacktestStrategyError { source, .. } => {
                    format!("回测策略错误: {}", source.get_error_message(language))
                }
                StrategyEngineError::UnsupportedStrategyType { strategy_type, .. } => {
                    format!("不支持的策略类型: {}", strategy_type)
                }
                StrategyEngineError::StrategyIsExist { strategy_id, .. } => {
                    format!("策略 {} 已存在", strategy_id)
                }
                StrategyEngineError::StrategyInstanceNotFound { strategy_id, .. } => {
                    format!("策略实例 {} 不存在", strategy_id)
                }
                StrategyEngineError::Database { source, .. } => {
                    format!("数据库错误: {}", source)
                }
                StrategyEngineError::UnsupportedTradeMode { trade_mode, .. } => {
                    format!("不支持的交易模式: {}", trade_mode)
                }
                StrategyEngineError::StrategyConfigNotFound { strategy_id, source, .. } => {
                    format!("策略 {} 配置不存在: {}", strategy_id, source)
                }
            },
        }
    }
}
