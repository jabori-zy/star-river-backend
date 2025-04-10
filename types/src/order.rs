use crate::market::Exchange;
use strum::{EnumString, Display};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
pub enum OrderStatus {
    #[strum(serialize = "created")]
    Created,
    #[strum(serialize = "placed")]
    Placed,
    #[strum(serialize = "filled")]
    Filled,
    #[strum(serialize = "partial")]
    Partial,
    #[strum(serialize = "canceled")]
    Canceled,
    #[strum(serialize = "expired")]
    Expired,
    #[strum(serialize = "rejected")]
    Rejected,
}

#[derive(Debug, Serialize)]
pub struct Mt5OrderRequest {
    pub order_type: String,
    pub order_side: String,
    pub symbol: String,
    pub volume: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,

}


//订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub strategy_id: i32,
    pub node_id: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}

impl From<OrderRequest> for Mt5OrderRequest {
    fn from(value: OrderRequest) -> Self {
        Mt5OrderRequest {
            order_type: value.order_type.to_string(),
            order_side: value.order_side.to_string(),
            symbol: value.symbol,
            volume: value.quantity,
            price: value.price,
            tp: value.tp,
            sl: value.sl,
        }
    }

}



//订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub strategy_id: i64,
    pub node_id: String,
    pub order_id: i64,
    pub exchange: Exchange,
    pub symbol: String,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub order_status: OrderStatus,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}
