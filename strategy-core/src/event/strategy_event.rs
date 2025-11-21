use chrono::{DateTime, Utc};
use derive_more::From;
use serde::Serialize;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use strum::Display;
use utoipa::ToSchema;
use star_river_core::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};

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
    pub fn info(strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, message: String) -> Self {
        Self::Info(
            StrategyStateInfoLog {
                strategy_id,
                strategy_name,
                strategy_state,
                strategy_state_action,
                message,
                datetime: Utc::now(),
            }
        )
    }

    pub fn warn(strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, message: String, datetime: DateTime<Utc>) -> Self {
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
        strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, message: String) -> Self {
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
    pub fn new(strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, message: String, datetime: DateTime<Utc>) -> Self {
        Self { strategy_id, strategy_name, strategy_state, strategy_state_action, message, datetime }
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
    pub fn new(strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, message: String) -> Self {
        Self { strategy_id, strategy_name, strategy_state, strategy_state_action, error_code, error_code_chain, message, datetime: Utc::now() }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStateErrorLog {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    pub strategy_state: String,
    pub strategy_state_action: String,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub message: String,
    pub datetime: DateTime<Utc>,
}


impl StrategyStateErrorLog {
    pub fn new(strategy_id: StrategyId, strategy_name: String, strategy_state: String, strategy_state_action: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, message: String) -> Self {
        Self { 
            strategy_id, 
            strategy_name, 
            strategy_state, 
            strategy_state_action, 
            error_code, 
            error_code_chain, 
            message, 
            datetime: Utc::now() 
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
    pub fn info_with_time(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, detail: serde_json::Value, datetime: DateTime<Utc>) -> Self {
        Self::Info(StrategyRunningInfoLog::new(strategy_id, node_id, node_name, source, log_type, message, detail, Some(datetime)))
    }

    pub fn warn_with_time(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value, datetime: DateTime<Utc>) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, Some(datetime)))
    }

    pub fn error_with_time(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value, datetime: DateTime<Utc>) -> Self {
        Self::Error(StrategyRunningErrorLog::new(strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, Some(datetime)))
    }

    pub fn info(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, detail: serde_json::Value) -> Self {
        Self::Info(StrategyRunningInfoLog::new(strategy_id, node_id, node_name, source, log_type, message, detail, None))
    }

    pub fn warn(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value) -> Self {
        Self::Warn(StrategyRunningWarnLog::new(strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, None))
    }

    pub fn error(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value) -> Self {
        Self::Error(StrategyRunningErrorLog::new(strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, None))
    }
}


#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningInfoLog {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub source: StrategyRunningLogSource,
    pub log_type: StrategyRunningLogType,
    pub message: String,
    pub detail: serde_json::Value,
    pub datetime: DateTime<Utc>,
}


impl StrategyRunningInfoLog {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, detail: serde_json::Value, datetime: Option<DateTime<Utc>>) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self { strategy_id, node_id, node_name, source, log_type, message, detail, datetime }
    }
}


#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningWarnLog {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub source: StrategyRunningLogSource,
    pub log_type: StrategyRunningLogType,
    pub message: String,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub detail: serde_json::Value,
    pub datetime: DateTime<Utc>,
}


impl StrategyRunningWarnLog {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value, datetime: Option<DateTime<Utc>>) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self { strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, datetime }
    }


    pub fn new_with_error(
        strategy_id: StrategyId, 
        node_id: NodeId, 
        node_name: NodeName, 
        source: StrategyRunningLogSource, 
        log_type: StrategyRunningLogType, 
        language: Option<ErrorLanguage>,
        message: Option<String>,
        error: Option<&impl StarRiverErrorTrait>, 
        detail: serde_json::Value, 
        datetime: Option<DateTime<Utc>>
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
            strategy_id, 
            node_id, 
            node_name, 
            source, 
            log_type, 
            message: final_message,
            error_code, 
            error_code_chain, 
            detail, 
            datetime 
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunningErrorLog {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub source: StrategyRunningLogSource,
    pub log_type: StrategyRunningLogType,
    pub message: String,
    pub detail: serde_json::Value,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub datetime: DateTime<Utc>,
}


impl StrategyRunningErrorLog {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>, detail: serde_json::Value, datetime: Option<DateTime<Utc>>    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self { strategy_id, node_id, node_name, source, log_type, message, error_code, error_code_chain, detail, datetime }
    }

    pub fn new_with_error(strategy_id: StrategyId, node_id: NodeId, node_name: NodeName, source: StrategyRunningLogSource, log_type: StrategyRunningLogType, language: ErrorLanguage, error: &impl StarRiverErrorTrait, detail: serde_json::Value, datetime: Option<DateTime<Utc>>) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self { strategy_id, node_id, node_name, source, log_type, message: error.error_message(language), error_code: Some(error.error_code().to_string()), error_code_chain: Some(error.error_code_chain()), detail, datetime }
    }
}