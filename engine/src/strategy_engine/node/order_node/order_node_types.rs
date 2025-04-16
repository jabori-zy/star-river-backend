use types::order::{OrderType, OrderSide};


#[derive(Debug, Clone)]
pub struct OrderConfig {
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub comment: String,
}