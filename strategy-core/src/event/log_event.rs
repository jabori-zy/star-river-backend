use chrono::{DateTime, Utc};
use derive_more::From;
use serde::Serialize;
use utoipa::ToSchema;
use star_river_core::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};



#[derive(Debug, Clone, Serialize, ToSchema, From)]
#[serde(tag = "logLevel")]
pub enum NodeStateLogEvent {
    Info(NodeStateInfoLog),
    Warn(NodeStateWarnLog),
    Error(NodeStateErrorLog),
}





impl NodeStateLogEvent {
    pub fn info(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, message: String) -> Self {
        Self::Info(NodeStateInfoLog::new(strategy_id, node_id, node_name, node_type, node_state, node_state_action, message))
    }
    pub fn warn(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>) -> Self {
        Self::Warn(NodeStateWarnLog::new(strategy_id, node_id, node_name, node_type, node_state, node_state_action, message, error_code, error_code_chain))
    }
    pub fn error(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, error: &impl StarRiverErrorTrait) -> Self {
        Self::Error(NodeStateErrorLog::new(strategy_id, node_id, node_name, node_type,node_state, node_state_action, error))
    }
}





#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeStateInfoLog {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub node_state: String,
    pub node_state_action: String,
    pub message: String,
    pub datetime: DateTime<Utc>,
}

impl NodeStateInfoLog {
    pub fn new(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, message: String) -> Self {
        Self { strategy_id, node_id, node_name, node_type, node_state, node_state_action, message, datetime: Utc::now() }
    }
}


#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeStateWarnLog {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub node_state: String,
    pub node_state_action: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,
    pub datetime: DateTime<Utc>,
}


impl NodeStateWarnLog {
    pub fn new(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, message: String, error_code: Option<String>, error_code_chain: Option<Vec<String>>) -> Self {
        Self { strategy_id, node_id, node_name, node_type, node_state, node_state_action, message, error_code, error_code_chain, datetime: Utc::now() }
    }
}



#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeStateErrorLog {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub node_state: String,
    pub node_state_action: String,
    pub message: String,
    pub error_code: String,
    pub error_code_chain: Vec<String>,
    pub datetime: DateTime<Utc>,
}


impl NodeStateErrorLog {
    pub fn new(strategy_id: i32, node_id: String, node_name: String, node_type: String, node_state: String, node_state_action: String, error: &impl StarRiverErrorTrait) -> Self {
        let message = error.error_message(ErrorLanguage::Chinese);
        let error_code = error.error_code().to_string();
        let error_code_chain = error.error_code_chain();
        Self { strategy_id, node_id, node_name, node_type, node_state, node_state_action, message, error_code, error_code_chain, datetime: Utc::now() }
    }
}









// #[derive(Debug, Clone, Serialize, Deserialize, From)]
// #[serde(rename_all = "camelCase")]
// pub struct NodeStateLogEvent {
//     pub strategy_id: i32,

//     pub node_id: String,

//     pub node_name: String,

//     pub node_state: String,

//     pub node_state_action: String,

//     pub log_level: LogLevel,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error_code: Option<String>,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error_code_chain: Option<Vec<String>>,

//     pub message: String,
//     pub datetime: DateTime<Utc>,
// }

// impl NodeStateLogEvent {
//     pub fn success(
//         strategy_id: i32,
//         node_id: String,
//         node_name: String,
//         node_state: String,
//         node_state_action: String,
//         message: String,
//     ) -> Self {
//         Self {
//             strategy_id,
//             node_id,
//             node_name,
//             node_state,
//             node_state_action,
//             log_level: LogLevel::Info,
//             message,
//             error_code: None,
//             error_code_chain: None,
//             datetime: Utc::now(),
//         }
//     }

//     pub fn error(
//         strategy_id: i32,
//         node_id: String,
//         node_name: String,
//         node_state: String,
//         node_state_action: String,
//         error: &impl StarRiverErrorTrait,
//     ) -> Self {
//         Self {
//             strategy_id,
//             node_id,
//             node_name,
//             node_state,
//             node_state_action,
//             log_level: LogLevel::Error,
//             message: error.error_message(ErrorLanguage::Chinese),
//             error_code: Some(error.error_code().to_string()),
//             error_code_chain: Some(error.error_code_chain()),
//             datetime: Utc::now(),
//         }
//     }
// }

