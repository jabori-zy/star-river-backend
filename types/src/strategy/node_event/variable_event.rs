use serde::{Deserialize, Serialize};
use crate::market::Exchange;


// 仓位数量更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumberUpdateEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub from_node_handle_id: String,
    pub exchange: Option<Exchange>,
    pub symbol: Option<String>,
    pub position_number: u32,
    pub event_timestamp: i64,
}
