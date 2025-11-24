use chrono::{DateTime, Utc};
use derive_more::From;
use serde::Serialize;
use star_river_core::{
    custom_type::{CycleId, HandleId, NodeId, NodeName, StrategyId},
    error::error_trait::{ErrorLanguage, StarRiverErrorTrait},
};
use strum::Display;
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

#[derive(Debug, Clone, Serialize, Display, ToSchema)]
pub enum StrategyRunningLogSource {
    #[strum(serialize = "node")]
    #[serde(rename = "Node")]
    Node,
    #[strum(serialize = "virtual_trading_system")]
    #[serde(rename = "VirtualTradingSystem")]
    VirtualTradingSystem,
}

#[derive(Debug, Clone, Serialize, Display, ToSchema)]
pub enum StrategyRunningLogType {
    #[strum(serialize = "condition_match")]
    #[serde(rename = "ConditionMatch")]
    ConditionMatch,
    #[strum(serialize = "order_created")]
    #[serde(rename = "OrderCreated")]
    OrderCreated,
    #[strum(serialize = "order_filled")]
    #[serde(rename = "OrderFilled")]
    OrderFilled,
    #[strum(serialize = "order_canceled")]
    #[serde(rename = "OrderCanceled")]
    OrderCanceled,
    #[strum(serialize = "processing_order")]
    #[serde(rename = "ProcessingOrder")]
    ProcessingOrder,
}

#[derive(Debug, Clone, Serialize, ToSchema, From)]
#[serde(rename_all = "camelCase")]
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

    pub fn node_id(&self) -> &NodeId {
        match self {
            StrategyRunningLogEvent::Info(event) => &event.node_id,
            StrategyRunningLogEvent::Warn(event) => &event.node_id,
            StrategyRunningLogEvent::Error(event) => &event.node_id,
        }
    }
    pub fn node_name(&self) -> &NodeName {
        match self {
            StrategyRunningLogEvent::Info(event) => &event.node_name,
            StrategyRunningLogEvent::Warn(event) => &event.node_name,
            StrategyRunningLogEvent::Error(event) => &event.node_name,
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            StrategyRunningLogEvent::Info(event) => &event.output_handle_id,
            StrategyRunningLogEvent::Warn(event) => &event.output_handle_id,
            StrategyRunningLogEvent::Error(event) => &event.output_handle_id,
        }
    }
}

impl StrategyRunningLogEvent {
    pub fn info_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Info(StrategyRunningInfoLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            log_type,
            message,
            detail,
            Some(datetime),
        ))
    }

    pub fn warn_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            message,
            error_code,
            error_code_chain,
            Some(datetime),
        ))
    }

    pub fn error_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        error: &impl StarRiverErrorTrait,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Error(StrategyRunningErrorLog::new_with_time(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            ErrorLanguage::English,
            error,
            datetime,
        ))
    }

    pub fn info(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        log_type: StrategyRunningLogType,
        source: StrategyRunningLogSource,
        message: String,
        detail: serde_json::Value,
    ) -> Self {
        Self::Info(StrategyRunningInfoLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            log_type,
            message,
            detail,
            None,
        ))
    }

    pub fn warn(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
    ) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            message,
            error_code,
            error_code_chain,
            None,
        ))
    }

    pub fn error(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        Self::Error(StrategyRunningErrorLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
            source,
            error,
            None,
        ))
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningInfoLog {
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub output_handle_id: HandleId,
    pub source: StrategyRunningLogSource,
    pub log_type: StrategyRunningLogType,
    pub message: String,
    pub detail: serde_json::Value,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningInfoLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self {
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            source,
            log_type,
            message,
            detail,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningWarnLog {
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub source: StrategyRunningLogSource,
    pub output_handle_id: HandleId,
    pub message: String,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningWarnLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self {
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            source,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            message,
            error_code,
            error_code_chain,
            datetime,
        }
    }

    pub fn new_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        language: Option<ErrorLanguage>,
        source: StrategyRunningLogSource,
        message: Option<String>,
        error: Option<&impl StarRiverErrorTrait>,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());

        // 根据优先级确定 message: message > error.error_message() > ""
        let final_message = if let Some(msg) = message {
            msg
        } else if let Some(err) = error {
            err.error_message(language.unwrap_or(ErrorLanguage::English))
        } else {
            String::new()
        };

        // 如果 error 存在，提取 error_code 和 error_code_chain
        let (error_code, error_code_chain) = if let Some(err) = error {
            (Some(err.error_code().to_string()), Some(err.error_code_chain()))
        } else {
            (None, None)
        };

        Self {
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            source,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            message: final_message,
            error_code,
            error_code_chain,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningErrorLog {
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub output_handle_id: HandleId,
    pub source: StrategyRunningLogSource,
    pub message: String,
    pub error_code: String,
    pub error_code_chain: Vec<String>,
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningErrorLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        error: &impl StarRiverErrorTrait,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        let message = error.error_message(ErrorLanguage::Chinese);
        let error_code = error.error_code().to_string();
        let error_code_chain = error.error_code_chain();
        Self {
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            source,
            node_name,
            message,
            error_code,
            error_code_chain,
            datetime,
        }
    }

    pub fn new_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        source: StrategyRunningLogSource,
        language: ErrorLanguage,
        error: &impl StarRiverErrorTrait,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            source,
            message: error.error_message(language),
            error_code: error.error_code().to_string(),
            error_code_chain: error.error_code_chain(),
            datetime,
        }
    }
}
