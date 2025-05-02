use crate::market::Exchange;
use strum::{EnumString, Display};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
pub enum OrderSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
pub enum OrderType {
    #[strum(serialize = "market")]
    Market,
    #[strum(serialize = "limit")]
    Limit,
    #[strum(serialize = "stop")]
    Stop,
    #[strum(serialize = "stop_limit")]
    StopLimit
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
pub enum OrderStatus {
    #[strum(serialize = "created")] // 已创建
    Created,
    #[strum(serialize = "placed")] // 已挂单
    Placed,
    #[strum(serialize = "filled")] // 已成交
    Filled,
    #[strum(serialize = "partial")] // 部分成交
    Partial,
    #[strum(serialize = "canceled")] // 已取消
    Canceled,
    #[strum(serialize = "expired")] // 已过期
    Expired,
    #[strum(serialize = "rejected")] // 已拒绝
    Rejected,
}



// 系统级别的订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i64, // 订单ID
    pub strategy_id: i64, // 策略ID
    pub node_id: String, // 节点ID
    pub exchange_order_id: i64, // 交易所订单ID
    pub account_id: i32, // 账户ID
    pub exchange: Exchange, // 交易所
    pub symbol: String, // 交易对
    pub order_side: OrderSide, // 订单方向
    pub order_type: OrderType, // 订单类型
    pub order_status: OrderStatus, // 订单状态
    pub quantity: f64, // 数量
    pub open_price: f64, // 开仓价格
    pub tp: Option<f64>, // 止盈价格
    pub sl: Option<f64>, // 止损价格
    pub extra_info: Option<serde_json::Value>, // 额外信息
    pub created_time: DateTime<Utc>, // 创建时间
    pub updated_time: DateTime<Utc>, // 更新时间

}

pub trait OriginalOrder: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalOrder>;
    fn get_exchange_order_id(&self) -> i64;
    fn get_exchange(&self) -> Exchange;
    fn get_symbol(&self) -> String;
    fn get_order_side(&self) -> OrderSide;
    fn get_order_type(&self) -> OrderType;
    fn get_order_status(&self) -> OrderStatus;
    fn get_quantity(&self) -> f64;
    fn get_open_price(&self) -> f64;
    fn get_tp(&self) -> Option<f64>;
    fn get_sl(&self) -> Option<f64>;
    fn get_extra_info(&self) -> Option<serde_json::Value>;
    fn get_created_time(&self) -> DateTime<Utc>;
    fn get_updated_time(&self) -> DateTime<Utc>;


}

impl Clone for Box<dyn OriginalOrder> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


