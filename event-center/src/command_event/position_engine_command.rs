use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use uuid::Uuid;
use types::position::PositionNumberRequest;
use types::market::Exchange;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum PositionEngineCommand {
    #[strum(serialize = "create-order")]
    GetPositionNumber(GetPositionNumberParam),
    #[strum(serialize = "get-position")]
    GetPosition(GetPositionParam),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParam {
    pub strategy_id: i64,
    pub node_id: String,
    pub position_number_request: PositionNumberRequest,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionParam {
    pub strategy_id: i64,
    pub node_id: String,
    pub exchange: Exchange,
    pub position_id: i64,
}



