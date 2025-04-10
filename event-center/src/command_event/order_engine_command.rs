use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use uuid::Uuid;
use types::order::OrderRequest;




#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum OrderEngineCommand {
    #[strum(serialize = "create-order")]
    CreateOrder(CreateOrderParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderParams {
    pub strategy_id: i32,
    pub node_id: String,
    pub order_request: OrderRequest,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}