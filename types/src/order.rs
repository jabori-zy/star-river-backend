use crate::market::Exchange;
use strum::EnumString;
use strum::Display;
use serde::{Serialize, Deserialize};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
pub enum OrderStatus {
    #[strum(serialize = "open")]
    Open,
    #[strum(serialize = "closed")]
    Closed,
    #[strum(serialize = "canceled")]
    Canceled,
    #[strum(serialize = "expired")]
    Expired,
    #[strum(serialize = "rejected")]
    Rejected,
}


//订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub exchange: Exchange,
    pub symbol: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub comment: Option<String>,
}

//订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i64,
    pub exchange: Exchange,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}
