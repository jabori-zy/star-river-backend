use star_river_core::error::{StarRiverErrorTrait, ErrorCode, ErrorLanguage, generate_error_code_chain};
use sea_orm::error::DbErr;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use engine_core::state_machine_error::EngineStateMachineError;
use crate::error::strategy_error::BacktestStrategyError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum BacktestEngineError {

    #[snafu(transparent)]
    BacktestStrategyError {
        source: BacktestStrategyError,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    EngineStateMachineError {
        source: EngineStateMachineError,
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
impl StarRiverErrorTrait for BacktestEngineError {
    fn get_prefix(&self) -> &'static str {
        "BACKTEST_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // For nested errors, delegate to the inner error's code
            // StrategyEngineError::BacktestStrategyError { .. } => 1001,
            BacktestEngineError::BacktestStrategyError { .. } => 1001,
            BacktestEngineError::EngineStateMachineError { .. } => 1002,
            BacktestEngineError::UnsupportedStrategyType { .. } => 1003,
            BacktestEngineError::StrategyIsExist { .. } => 1004,
            BacktestEngineError::StrategyInstanceNotFound { .. } => 1005,
            BacktestEngineError::Database { .. } => 1006,
            BacktestEngineError::UnsupportedTradeMode { .. } => 1007,
            BacktestEngineError::StrategyConfigNotFound { .. } => 1008,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            BacktestEngineError::BacktestStrategyError { source ,..} => generate_error_code_chain(source),
            BacktestEngineError::EngineStateMachineError { source ,..} => generate_error_code_chain(source),

            _ => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                BacktestEngineError::EngineStateMachineError { source, .. } => source.error_message(language),
                BacktestEngineError::BacktestStrategyError { source, .. } => source.error_message(language),
                BacktestEngineError::UnsupportedStrategyType { strategy_type, .. } => {
                    format!("不支持的策略类型: {}", strategy_type)
                }
                BacktestEngineError::StrategyIsExist { strategy_id, .. } => {
                    format!("策略 {} 已存在", strategy_id)
                }
                BacktestEngineError::StrategyInstanceNotFound { strategy_id, .. } => {
                    format!("策略实例 {} 不存在", strategy_id)
                }
                BacktestEngineError::Database { source, .. } => {
                    format!("数据库错误: {}", source)
                }
                BacktestEngineError::UnsupportedTradeMode { trade_mode, .. } => {
                    format!("不支持的交易模式: {}", trade_mode)
                }
                BacktestEngineError::StrategyConfigNotFound { strategy_id, source, .. } => {
                    format!("策略 {} 配置不存在: {}", strategy_id, source)
                }
            },
        }
    }
}
