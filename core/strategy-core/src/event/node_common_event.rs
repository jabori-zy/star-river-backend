use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize, Serializer};
use star_river_core::{
    custom_type::{CycleId, HandleId, NodeId, NodeName, StrategyId},
    error::error_trait::{ErrorLanguage, StarRiverErrorTrait},
};
use strum::Display;
use utoipa::ToSchema;

use crate::event::{log_event::NodeStateLogEvent, node::NodeEvent};

#[derive(Debug, Clone, Serialize, From)]
#[serde(tag = "event")]
pub enum CommonEvent {
    Trigger(TriggerEvent),               // Trigger event
    ExecuteOver(ExecuteOverEvent),       // Execute over
    NodeRunningLog(NodeRunningLogEvent), // Running log
    RunStateLog(NodeStateLogEvent),      // State log
}

impl CommonEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            CommonEvent::Trigger(event) => event.cycle_id(),
            CommonEvent::ExecuteOver(event) => event.cycle_id(),
            CommonEvent::NodeRunningLog(event) => event.cycle_id(),
            CommonEvent::RunStateLog(_) => 0,
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            CommonEvent::Trigger(event) => event.datetime(),
            CommonEvent::ExecuteOver(event) => event.datetime(),
            CommonEvent::NodeRunningLog(event) => event.datetime(),
            CommonEvent::RunStateLog(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            CommonEvent::Trigger(event) => event.node_id(),
            CommonEvent::ExecuteOver(event) => event.node_id(),
            CommonEvent::NodeRunningLog(event) => event.node_id(),
            CommonEvent::RunStateLog(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            CommonEvent::Trigger(event) => event.node_name(),
            CommonEvent::ExecuteOver(event) => event.node_name(),
            CommonEvent::NodeRunningLog(event) => event.node_name(),
            CommonEvent::RunStateLog(event) => event.node_name(),
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            CommonEvent::Trigger(event) => event.output_handle_id(),
            CommonEvent::ExecuteOver(event) => event.output_handle_id(),
            CommonEvent::NodeRunningLog(event) => event.output_handle_id(),
            CommonEvent::RunStateLog(event) => event.output_handle_id(),
        }
    }
}

// Type aliases

pub type TriggerEvent = NodeEvent<TriggerPayload>;
pub type ExecuteOverEvent = NodeEvent<ExecuteOverPayload>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteOverPayload {
    pub config_id: Option<i32>,
}

impl ExecuteOverPayload {
    pub fn new(config_id: Option<i32>) -> Self {
        Self { config_id }
    }
}

// #[derive(Debug, Clone, Serialize, Display, ToSchema)]
// pub enum NodeRunningLogSource {
//     #[strum(serialize = "node")]
//     #[serde(rename = "Node")]
//     Node,
//     #[strum(serialize = "virtual_trading_system")]
//     #[serde(rename = "VirtualTradingSystem")]
//     VirtualTradingSystem,
// }

// #[derive(Debug, Clone, Serialize, Display, ToSchema)]
// pub enum NodeRunningLogType {
//     #[strum(serialize = "condition_match")]
//     #[serde(rename = "ConditionMatch")]
//     ConditionMatch,
//     #[strum(serialize = "order_created")]
//     #[serde(rename = "OrderCreated")]
//     OrderCreated,
//     #[strum(serialize = "order_filled")]
//     #[serde(rename = "OrderFilled")]
//     OrderFilled,
//     #[strum(serialize = "order_canceled")]
//     #[serde(rename = "OrderCanceled")]
//     OrderCanceled,
//     #[strum(serialize = "processing_order")]
//     #[serde(rename = "ProcessingOrder")]
//     ProcessingOrder,
// }

#[derive(Debug, Clone, Serialize, ToSchema, From)]
#[serde(tag = "logLevel")]
pub enum NodeRunningLogEvent {
    Info(NodeRunningInfoLog),
    Warn(NodeRunningWarnLog),
    Error(NodeRunningErrorLog),
}

impl NodeRunningLogEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            NodeRunningLogEvent::Info(event) => event.cycle_id,
            NodeRunningLogEvent::Warn(event) => event.cycle_id,
            NodeRunningLogEvent::Error(event) => event.cycle_id,
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            NodeRunningLogEvent::Info(event) => event.datetime,
            NodeRunningLogEvent::Warn(event) => event.datetime,
            NodeRunningLogEvent::Error(event) => event.datetime,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            NodeRunningLogEvent::Info(event) => &event.node_id,
            NodeRunningLogEvent::Warn(event) => &event.node_id,
            NodeRunningLogEvent::Error(event) => &event.node_id,
        }
    }
    pub fn node_name(&self) -> &NodeName {
        match self {
            NodeRunningLogEvent::Info(event) => &event.node_name,
            NodeRunningLogEvent::Warn(event) => &event.node_name,
            NodeRunningLogEvent::Error(event) => &event.node_name,
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            NodeRunningLogEvent::Info(event) => &event.output_handle_id,
            NodeRunningLogEvent::Warn(event) => &event.output_handle_id,
            NodeRunningLogEvent::Error(event) => &event.output_handle_id,
        }
    }
}

impl NodeRunningLogEvent {
    pub fn info_with_time(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        message: String,
        detail: serde_json::Value,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Info(NodeRunningInfoLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
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
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Warn(NodeRunningWarnLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
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
        error: &impl StarRiverErrorTrait,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::Error(NodeRunningErrorLog::new_with_time(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
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
        message: String,
        detail: serde_json::Value,
    ) -> Self {
        Self::Info(NodeRunningInfoLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
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
        message: String,
        error_code: Option<String>,
        error_code_chain: Option<Vec<String>>,
    ) -> Self {
        Self::Warn(NodeRunningWarnLog::new(
            cycle_id,
            strategy_id,
            node_id,
            node_name,
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
        error: &impl StarRiverErrorTrait,
    ) -> Self {
        Self::Error(NodeRunningErrorLog::new(cycle_id, strategy_id, node_id, node_name, error, None))
    }
}

// Helper function to serialize type field as "node"
fn serialize_node_type<S>(_: &(), serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("node")
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeRunningInfoLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_node_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    #[serde(skip)]
    pub output_handle_id: HandleId,
    pub message: String,
    pub detail: serde_json::Value,
    pub datetime: DateTime<Utc>,
}

impl NodeRunningInfoLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        message: String,
        detail: serde_json::Value,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            message,
            detail,
            datetime,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeRunningWarnLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_node_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    #[serde(skip)]
    pub output_handle_id: HandleId,
    pub message: String,
    pub error_code: Option<String>,
    pub error_code_chain: Option<Vec<String>>,
    pub datetime: DateTime<Utc>,
}

impl NodeRunningWarnLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
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
            node_id: node_id.clone(),
            node_name,
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
            _type: (),
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
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
pub struct NodeRunningErrorLog {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_node_type")]
    #[serde(skip_deserializing)]
    _type: (),
    pub cycle_id: CycleId,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    #[serde(skip)]
    pub output_handle_id: HandleId,
    pub message: String,
    pub error_code: String,
    pub error_code_chain: Vec<String>,
    pub datetime: DateTime<Utc>,
}

impl NodeRunningErrorLog {
    pub fn new(
        cycle_id: CycleId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        error: &impl StarRiverErrorTrait,
        datetime: Option<DateTime<Utc>>,
    ) -> Self {
        let datetime = datetime.unwrap_or(Utc::now());
        let message = error.error_message(ErrorLanguage::Chinese);
        let error_code = error.error_code().to_string();
        let error_code_chain = error.error_code_chain();
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            output_handle_id: format!("{}_strategy_output_handle", node_id),
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
        language: ErrorLanguage,
        error: &impl StarRiverErrorTrait,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            _type: (),
            cycle_id,
            strategy_id,
            node_id: node_id.clone(),
            node_name,
            output_handle_id: format!("{}_strategy_output_handle", node_id),
            message: error.error_message(language),
            error_code: error.error_code().to_string(),
            error_code_chain: error.error_code_chain(),
            datetime,
        }
    }
}
