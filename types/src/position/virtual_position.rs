use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::custom_type::*;
use crate::market::Exchange;
use crate::position::{PositionSide, PositionState};




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPosition {
    pub position_id: i32,
    pub strategy_id: i64,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub exchange_position_id: i64,
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_state: PositionState, // 持仓状态
    pub quantity: f64,
    pub open_price: f64,
    pub current_price: Option<f64>,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub unrealized_profit: Option<f64>, // 未实现盈亏
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}