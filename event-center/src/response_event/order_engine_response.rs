
use types::order::Order;
use types::market::Exchange;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEngineResponse {
    // 创建订单响应
    CreateOrderResponse(CreateOrderResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub code: i32,
    pub message: String,
    pub response_timestamp: i64,
    pub response_id: Uuid,
    pub order: Option<Order>,
}