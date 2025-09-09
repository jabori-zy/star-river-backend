pub mod virtual_position;

use crate::market::Exchange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use strum::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParam {
    pub strategy_id: i32,
    pub node_id: String,
    pub position_number_request: GetPositionNumberParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionParam {
    pub strategy_id: i32,
    pub node_id: String,
    pub exchange: Exchange,
    pub position_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
// 订单方向
pub enum PositionSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum PositionState {
    #[strum(serialize = "open")]
    Open, // 持仓中
    #[strum(serialize = "closed")]
    Closed, // 已平仓
    #[strum(serialize = "partially_closed")]
    PartiallyClosed, // 部分平仓
    #[strum(serialize = "forced_closed")]
    ForcedClosed, // 强制平仓
}

// 交易所返回的原始仓位信息
pub trait OriginalPosition: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalPosition>;
    fn get_exchange_position_id(&self) -> i64;
    fn get_symbol(&self) -> String;
    fn get_position_side(&self) -> PositionSide;
    fn get_quantity(&self) -> f64;
    fn get_open_price(&self) -> f64;
    fn get_tp(&self) -> Option<f64>;
    fn get_sl(&self) -> Option<f64>;
    fn get_exchange(&self) -> Exchange;
    fn get_unrealized_profit(&self) -> Option<f64>;
    fn get_extra_info(&self) -> Option<serde_json::Value>;
    fn get_create_time(&self) -> DateTime<Utc>;
    fn get_update_time(&self) -> DateTime<Utc>;
}

impl Clone for Box<dyn OriginalPosition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
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
    pub unrealized_profit: Option<f64>,        // 未实现盈亏
    pub extra_info: Option<serde_json::Value>, // 额外信息
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

// 订单数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionNumberParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
}

// 仓位数量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumber {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
    pub position_number: i32,
}
