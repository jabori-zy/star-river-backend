pub mod virtual_order;

use crate::market::Exchange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use std::str::FromStr;
use strum::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderParams {
    #[serde(flatten)]
    pub strategy_id: i32,
    pub node_id: String,
    pub account_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub order_type: OrderType,
    pub order_side: FuturesOrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionDetailParams {
    pub strategy_id: i32,
    pub node_id: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub transaction_id: Option<i64>,
    pub position_id: Option<i64>,
    pub order_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum FuturesOrderSide {
    #[strum(serialize = "OPEN_LONG")]
    #[serde(rename = "OPEN_LONG")]
    OpenLong,
    #[strum(serialize = "OPEN_SHORT")]
    #[serde(rename = "OPEN_SHORT")]
    OpenShort,
    #[strum(serialize = "CLOSE_LONG")]
    #[serde(rename = "CLOSE_LONG")]
    CloseLong,
    #[strum(serialize = "CLOSE_SHORT")]
    #[serde(rename = "CLOSE_SHORT")]
    CloseShort,
}

// 止盈止损类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum TpslType {
    #[strum(serialize = "price")]
    #[serde(rename = "price")]
    Price,
    #[strum(serialize = "percentage")]
    #[serde(rename = "percentage")]
    Percentage,
}

pub fn deserialize_futures_order_side<'de, D>(deserializer: D) -> Result<FuturesOrderSide, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;

    // 使用as_str()方法获取&str，然后传递给from_str
    match FuturesOrderSide::from_str(s.as_str()) {
        Ok(order_side) => Ok(order_side),
        Err(e) => Err(serde::de::Error::custom(format!(
            "无法解析OrderSide: {}",
            e
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum OrderType {
    #[strum(serialize = "MARKET")]
    #[serde(rename = "MARKET")]
    Market,
    #[strum(serialize = "LIMIT")]
    #[serde(rename = "LIMIT")]
    Limit,
    #[strum(serialize = "STOP_MARKET")]
    #[serde(rename = "STOP_MARKET")]
    StopMarket,
    #[strum(serialize = "STOP_LIMIT")]
    #[serde(rename = "STOP_LIMIT")]
    StopLimit,
    #[strum(serialize = "TAKE_PROFIT_LIMIT")]
    #[serde(rename = "TAKE_PROFIT_LIMIT")]
    TakeProfitLimit,
    #[strum(serialize = "TAKE_PROFIT_MARKET")]
    #[serde(rename = "TAKE_PROFIT_MARKET")]
    TakeProfitMarket,
}

pub fn deserialize_order_type<'de, D>(deserializer: D) -> Result<OrderType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;

    // 使用as_str()方法获取&str，然后传递给from_str
    match OrderType::from_str(s.as_str()) {
        Ok(order_type) => Ok(order_type),
        Err(e) => Err(serde::de::Error::custom(format!(
            "无法解析OrderType: {}",
            e
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum OrderStatus {
    #[strum(serialize = "created")] // 已创建
    #[serde(rename = "CREATED")]
    Created,

    #[strum(serialize = "placed")] // 已挂单
    #[serde(rename = "PLACED")]
    Placed,

    #[strum(serialize = "filled")] // 已成交
    #[serde(rename = "FILLED")]
    Filled,

    #[strum(serialize = "partial")] // 部分成交
    #[serde(rename = "PARTIAL")]
    Partial,

    #[strum(serialize = "canceled")] // 已取消
    #[serde(rename = "CANCELED")]
    Canceled,

    #[strum(serialize = "expired")] // 已过期
    #[serde(rename = "EXPIRED")]
    Expired,

    #[strum(serialize = "rejected")] // 已拒绝
    #[serde(rename = "REJECTED")]
    Rejected,
}

// 系统级别的订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i32,                         // 订单ID
    pub strategy_id: i32,                      // 策略ID
    pub node_id: String,                       // 节点ID
    pub exchange_order_id: i64,                // 交易所订单ID
    pub account_id: i32,                       // 账户ID
    pub exchange: Exchange,                    // 交易所
    pub symbol: String,                        // 交易对
    pub order_side: FuturesOrderSide,          // 订单方向
    pub order_type: OrderType,                 // 订单类型
    pub order_status: OrderStatus,             // 订单状态
    pub quantity: f64,                         // 数量
    pub open_price: f64,                       // 开仓价格
    pub tp: Option<f64>,                       // 止盈价格
    pub sl: Option<f64>,                       // 止损价格
    pub extra_info: Option<serde_json::Value>, // 额外信息
    pub created_time: DateTime<Utc>,           // 创建时间
    pub updated_time: DateTime<Utc>,           // 更新时间
}

pub trait OriginalOrder: Debug + Send + Sync + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalOrder>;
    fn get_exchange_order_id(&self) -> i64;
    fn get_exchange(&self) -> Exchange;
    fn get_symbol(&self) -> String;
    fn get_order_side(&self) -> FuturesOrderSide;
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
