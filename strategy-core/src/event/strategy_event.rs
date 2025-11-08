use serde::Serialize;
use star_river_core::custom_type::StrategyId;
use crate::benchmark::strategy_benchmark::StrategyPerformanceReport;
use crate::event::log_event::LogLevel;
use chrono::{DateTime, Utc};

// 策略性能更新时间
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPerformanceUpdateEvent {
    pub strategy_id: StrategyId,
    pub report: StrategyPerformanceReport,
}

impl StrategyPerformanceUpdateEvent {
    pub fn new(strategy_id: StrategyId, report: StrategyPerformanceReport) -> Self {
        Self {
            strategy_id,
            report,
        }
    }
}






#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStateLogEvent {
    pub strategy_id: i32,

    pub strategy_name: String,

    pub strategy_state: Option<String>,

    pub strategy_state_action: Option<String>,

    pub log_level: LogLevel,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,

    pub message: String,

    pub datetime: DateTime<Utc>,
}

impl StrategyStateLogEvent {
    pub fn new(
        strategy_id: i32,
        strategy_name: String,
        strategy_state: Option<String>,
        strategy_state_action: Option<String>,
        log_level: LogLevel,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            log_level,
            error_code,
            error_code_chain,
            message,
            datetime: Utc::now(),
        }
    }
}