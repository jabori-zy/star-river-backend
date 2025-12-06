use std::{error::Error, sync::Arc};

use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::StrategyName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum StrategyStatsError {
    #[snafu(display("#[{strategy_name}] failed to send strategy stats event: {source}"))]
    SendEventFailed {
        strategy_name: StrategyName,
        source: Arc<dyn Error + Send + Sync + 'static>,
        backtrace: Backtrace,
    },
}

impl StarRiverErrorTrait for StrategyStatsError {
    fn get_prefix(&self) -> &'static str {
        "STRATEGY_STATS"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            StrategyStatsError::SendEventFailed { .. } => 1001,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                StrategyStatsError::SendEventFailed { strategy_name, source, .. } => {
                    format!("#[{strategy_name}] 发送策略统计事件失败: {}", source)
                }
            },
        }
    }
}
