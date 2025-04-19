use crate::market::Exchange;
use strum::{EnumString, Display};
use serde::{Serialize, Deserialize};
use std::any::Any;
use std::fmt::Debug;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
// 订单方向
pub enum PositionSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}


#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum PositionStatus {
    #[strum(serialize = "open")]
    Open,
    #[strum(serialize = "close")]
    Close,
}


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GetPositionRequest {
//     pub strategy_id: i64,
//     pub node_id: i64,
//     pub exchange: Exchange,
//     pub symbol: String,
//     pub position_id: i64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ExchangePosition {
//     pub exchange_position_id: i64,
//     pub symbol: String,
//     pub position_side: PositionSide,
//     pub volume: f64,
//     pub open_price: f64,
//     pub current_price: f64,
//     pub tp: f64,
//     pub sl: f64,
//     pub unrealized_profit: f64, // 未实现盈亏
//     pub status: PositionStatus,
//     pub create_time: i64,
//     pub update_time: i64,
// }

pub trait ExchangePosition: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn ExchangePosition>;
    fn get_exchange_position_id(&self) -> i64;
    fn get_symbol(&self) -> String;
    fn get_position_side(&self) -> PositionSide;
    fn get_quantity(&self) -> f64;
    fn get_open_price(&self) -> f64;
    fn get_tp(&self) -> Option<f64>;
    fn get_sl(&self) -> Option<f64>;
    fn get_exchange(&self) -> Exchange;
    fn get_create_time(&self) -> DateTime<Utc>;
    fn get_update_time(&self) -> DateTime<Utc>;
}


impl Clone for Box<dyn ExchangePosition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: i64,
    pub strategy_id: i64,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub exchange_position_id: i64,
    pub symbol: String,
    pub position_side: PositionSide,
    pub quantity: f64,
    pub open_price: f64,
    pub current_price: Option<f64>,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub unrealized_profit: Option<f64>, // 未实现盈亏
    pub create_time: i64,
    pub update_time: i64,
}










// 订单数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionNumberRequest {
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
    pub position_number: i32
}