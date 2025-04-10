use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use uuid::Uuid;
use types::position::PositionNumberRequest;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum PositionEngineCommand {
    #[strum(serialize = "create-order")]
    GetPositionNumber(GetPositionNumberParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParams {
    pub strategy_id: i32,
    pub node_id: String,
    pub position_number_request: PositionNumberRequest,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}





