use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Serialize, Serializer};
use star_river_core::{
    custom_type::{CycleId, StrategyId},
    error::error_trait::{ErrorLanguage, StarRiverErrorTrait},
};
use utoipa::ToSchema;

use crate::benchmark::strategy_benchmark::StrategyPerformanceReport;

// 策略性能更新时间
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPerformanceUpdateEvent {
    pub strategy_id: StrategyId,
    pub report: StrategyPerformanceReport,
}

impl StrategyPerformanceUpdateEvent {
    pub fn new(strategy_id: StrategyId, report: StrategyPerformanceReport) -> Self {
        Self { strategy_id, report }
    }
}

#[derive(Debug, Clone, Serialize, From)]
#[serde(tag = "logLevel")]
pub enum StrategyStateLogEvent {
    Info(StrategyStateInfoLog),
    Warn(StrategyStateWarnLog),
    Error(StrategyStateErrorLog),
}

impl StrategyStateLogEvent {
    pub fn info(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        message: String,
    ) -> Self {
        Self::Info(StrategyStateInfoLog {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            message,
            datetime: Utc::now(),
        })
    }

    pub fn warn(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        message: String,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Warn(StrategyStateWarnLog {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            error_code,
            error_code_chain,
            message,
            datetime,
        })
    }

    pub fn error(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        let error_code = error.error_code().to_string();
        let error_code_chain = error.error_code_chain();
        let message = error.error_message(ErrorLanguage::Chinese);
        Self::Error(StrategyStateErrorLog {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            error_code,
            error_code_chain,
            message,
            datetime: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStateInfoLog {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    pub strategy_state: String,
    pub strategy_state_action: String,
    pub message: String,
    pub datetime: DateTime<Utc>,
}

impl StrategyStateInfoLog {
    pub fn new(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        message: String,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            message,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStateWarnLog {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    pub strategy_state: String,
    pub strategy_state_action: String,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub message: String,
    pub datetime: DateTime<Utc>,
}

impl StrategyStateWarnLog {
    pub fn new(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            error_code,
            error_code_chain,
            message,
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStateErrorLog {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    pub strategy_state: String,
    pub strategy_state_action: String,
    pub error_code: String,
    pub error_code_chain: Vec<String>,
    pub message: String,
    pub datetime: DateTime<Utc>,
}

impl StrategyStateErrorLog {
    pub fn new(
        strategy_id: StrategyId,
        strategy_name: String,
        strategy_state: String,
        strategy_state_action: String,
        error_code: String,
        error_code_chain: Vec<String>,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name,
            strategy_state,
            strategy_state_action,
            error_code,
            error_code_chain,
            message,
            datetime: Utc::now(),
        }
    }
}

// Helper function to serialize type field as "strategy"
fn serialize_strategy_type<S>(_: &(), serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("strategy")
}

#[derive(Debug, Clone, Serialize, ToSchema, From)]
#[serde(tag = "logLevel")]
pub enum StrategyRunningLogEvent {
    Info(StrategyRunningInfoLog),
    Warn(StrategyRunningWarnLog),
    Error(StrategyRunningErrorLog),
}

impl StrategyRunningLogEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            StrategyRunningLogEvent::Info(event) => event.cycle_id,
            StrategyRunningLogEvent::Warn(event) => event.cycle_id,
            StrategyRunningLogEvent::Error(event) => event.cycle_id,
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            StrategyRunningLogEvent::Info(event) => event.datetime,
            StrategyRunningLogEvent::Warn(event) => event.datetime,
            StrategyRunningLogEvent::Error(event) => event.datetime,
        }
    }

    pub fn strategy_id(&self) -> &StrategyId {
        match self {
            StrategyRunningLogEvent::Info(event) => &event.strategy_id,
            StrategyRunningLogEvent::Warn(event) => &event.strategy_id,
            StrategyRunningLogEvent::Error(event) => &event.strategy_id,
        }
    }
}

impl StrategyRunningLogEvent {
    pub fn info_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        message: String,
        detail: serde_json::Value,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Info(StrategyRunningInfoLog::new(cycle_id, strategy_id, message, detail, Some(datetime)))
    }

    pub fn warn_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(
            cycle_id,
            strategy_id,
            message,
            error_code,
            error_code_chain,
            Some(datetime),
        ))
    }

    pub fn error_with_time(cycle_id: CycleId, strategy_id: StrategyId, error: &impl StarRiverErrorTrait, datetime: DateTime<Utc>) -> Self {
        Self::Error(StrategyRunningErrorLog::new_with_time(
            cycle_id,
            strategy_id,
            ErrorLanguage::English,
            error,
            datetime,
        ))
    }

    pub fn info(cycle_id: CycleId, strategy_id: StrategyId, message: String, detail: serde_json::Value) -> Self {
        Self::Info(StrategyRunningInfoLog::new(cycle_id, strategy_id, message, detail, None))
    }

    pub fn warn(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
    ) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(
            cycle_id,
            strategy_id,
            message,
            error_code,
            error_code_chain,
            None,
        ))
    }

    pub fn error(cycle_id: CycleId, strategy_id: StrategyId, error: &impl StarRiverErrorTrait) -> Self {
        Self::Error(StrategyRunningErrorLog::new(cycle_id, strategy_id, error, None))
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningInfoLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_strategy_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub message: String,
    pub detail: serde_json::Value,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningInfoLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        message: String,
        detail: serde_json::Value,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            message,
            detail,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningWarnLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_strategy_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub message: String,
    pub detail: serde_json::Value,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningWarnLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            message,
            detail: serde_json::Value::Null,
            error_code,
            error_code_chain,
            datetime,
        }
    }

    pub fn new_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        language: Option<ErrorLanguage>,
        message: Option<String>,
        error: Option<&impl StarRiverErrorTrait>,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());

        // Determine message priority: message > error.error_message() > ""
        let final_message = if let Some(msg) = message {
            msg
        } else if let Some(err) = error {
            err.error_message(language.unwrap_or(ErrorLanguage::English))
        } else {
            String::new()
        };

        // Extract error_code and error_code_chain if error exists
        let (error_code, error_code_chain) = if let Some(err) = error {
            (Some(err.error_code().to_string()), Some(err.error_code_chain()))
        } else {
            (None, None)
        };

        Self {
            _type: (),
            cycle_id,
            strategy_id,
            message: final_message,
            detail: serde_json::Value::Null,
            error_code,
            error_code_chain,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningErrorLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_strategy_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub message: String,
    pub detail: serde_json::Value,
    pub error_code: String,
    pub error_code_chain: Vec<String>,
    pub report: String,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningErrorLog {
    pub fn new(cycle_id: CycleId, strategy_id: StrategyId, error: &impl StarRiverErrorTrait, datetime: Option<DateTime<Utc>>) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        let message = error.error_message(ErrorLanguage::English);
        let error_code = error.error_code().to_string();
        let error_code_chain = error.error_code_chain();
        let report = error.report();
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            message,
            detail: serde_json::Value::Null,
            error_code,
            error_code_chain,
            report,
            datetime,
        }
    }

    #[allow(unused)]
    pub fn new_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        language: ErrorLanguage,
        error: &impl StarRiverErrorTrait,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::new(cycle_id, strategy_id, error, Some(datetime))
    }
}
