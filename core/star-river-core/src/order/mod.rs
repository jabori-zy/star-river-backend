// pub mod virtual_order;

use std::{any::Any, fmt::Debug, str::FromStr};

use entity::order::Model as OrderModel;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::{exchange::Exchange, system::DateTimeUtc};

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
    #[strum(serialize = "LONG")]
    #[serde(rename = "LONG")]
    Long,
    #[strum(serialize = "SHORT")]
    #[serde(rename = "SHORT")]
    Short,
    // #[strum(serialize = "CLOSE LONG")]
    // #[serde(rename = "CLOSE_LONG")]
    // CloseLong,
    // #[strum(serialize = "CLOSE SHORT")]
    // #[serde(rename = "CLOSE_SHORT")]
    // CloseShort,
}

// Take profit and stop loss type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum TpslType {
    #[strum(serialize = "price")]
    #[serde(rename = "price")]
    Price,

    #[strum(serialize = "percentage")]
    #[serde(rename = "percentage")]
    Percentage,

    #[strum(serialize = "point")]
    #[serde(rename = "point")]
    Point,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum OrderStatus {
    #[strum(serialize = "created")] // Created
    #[serde(rename = "CREATED")]
    Created,

    #[strum(serialize = "placed")] // Placed
    #[serde(rename = "PLACED")]
    Placed,

    #[strum(serialize = "filled")] // Filled
    #[serde(rename = "FILLED")]
    Filled,

    #[strum(serialize = "partial")] // Partially filled
    #[serde(rename = "PARTIAL")]
    Partial,

    #[strum(serialize = "canceled")] // Canceled
    #[serde(rename = "CANCELED")]
    Canceled,

    #[strum(serialize = "expired")] // Expired
    #[serde(rename = "EXPIRED")]
    Expired,

    #[strum(serialize = "rejected")] // Rejected
    #[serde(rename = "REJECTED")]
    Rejected,
}

// System-level order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i32,                         // Order ID
    pub strategy_id: i32,                      // Strategy ID
    pub node_id: String,                       // Node ID
    pub exchange_order_id: i64,                // Exchange order ID
    pub account_id: i32,                       // Account ID
    pub exchange: Exchange,                    // Exchange
    pub symbol: String,                        // Trading symbol
    pub order_side: FuturesOrderSide,          // Order side
    pub order_type: OrderType,                 // Order type
    pub order_status: OrderStatus,             // Order status
    pub quantity: f64,                         // Quantity
    pub open_price: f64,                       // Open price
    pub tp: Option<f64>,                       // Take profit price
    pub sl: Option<f64>,                       // Stop loss price
    pub extra_info: Option<serde_json::Value>, // Extra info
    pub created_time: DateTimeUtc,             // Created time
    pub updated_time: DateTimeUtc,             // Updated time
}

impl From<OrderModel> for Order {
    fn from(model: OrderModel) -> Self {
        Self {
            order_id: model.id,
            strategy_id: model.strategy_id as i32,
            node_id: model.node_id,
            exchange_order_id: model.exchange_order_id,
            account_id: model.account_id,
            exchange: Exchange::from_str(&model.exchange).unwrap(),
            symbol: model.symbol,
            order_side: FuturesOrderSide::from_str(&model.order_side).unwrap(),
            order_type: OrderType::from_str(&model.order_type).unwrap(),
            order_status: OrderStatus::from_str(&model.order_status).unwrap(),
            quantity: model.quantity,
            open_price: model.price,
            tp: model.tp,
            sl: model.sl,
            extra_info: model.extra_info,
            created_time: model.created_time,
            updated_time: model.updated_time,
        }
    }
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
    fn get_created_time(&self) -> DateTimeUtc;
    fn get_updated_time(&self) -> DateTimeUtc;
}

impl Clone for Box<dyn OriginalOrder> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
