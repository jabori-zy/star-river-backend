use derive_more::From;
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use star_river_core::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};


#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[serde(rename_all = "camelCase")]
pub struct NodeStateLogEvent {
    pub strategy_id: i32,

    pub node_id: String,

    pub node_name: String,

    pub node_state: String,

    pub node_state_action: String,

    pub log_level: LogLevel,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,

    pub message: String,
    pub datetime: DateTime<Utc>,
}

impl NodeStateLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        message: String,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Info,
            message,
            error_code: None,
            error_code_chain: None,
            datetime: Utc::now(),
        }
    }

    pub fn error(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        node_state: String,
        node_state_action: String,
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            node_state,
            node_state_action,
            log_level: LogLevel::Error,
            message: error.error_message(ErrorLanguage::Chinese),
            error_code: Some(error.error_code().to_string()),
            error_code_chain: Some(error.error_code_chain()),
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
pub enum StrategyRunningLogSource {
    #[strum(serialize = "node")]
    #[serde(rename = "Node")]
    Node,
    #[strum(serialize = "virtual_trading_system")]
    #[serde(rename = "VirtualTradingSystem")]
    VirtualTradingSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
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

// 策略运行日志
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, From)]
pub struct StrategyRunningLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "nodeId")]
    pub node_id: String,

    #[serde(rename = "nodeName")]
    pub node_name: String,

    #[serde(rename = "source")]
    pub source: StrategyRunningLogSource,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "logType")]
    pub log_type: StrategyRunningLogType,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "detail")]
    pub detail: serde_json::Value,

    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "datetime")]
    #[schema(value_type = String, example = "2024-01-01T12:00:00Z")]
    pub datetime: DateTime<Utc>,
}

impl StrategyRunningLogEvent {
    pub fn success(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            source,
            log_level: LogLevel::Info,
            log_type,
            message,
            detail,
            error_code: None,
            error_code_chain: None,
            datetime,
        }
    }

    pub fn warn(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        source: StrategyRunningLogSource,
        log_type: StrategyRunningLogType,
        message: String,
        detail: serde_json::Value,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            source,
            log_level: LogLevel::Warn,
            log_type,
            message,
            detail,
            error_code: None,
            error_code_chain: None,
            datetime,
        }
    }
}