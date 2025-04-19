use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::order::{OrderSide, OrderType};
use types::market::Exchange;
use crate::command_event::BaseCommandParams;




#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum OrderEngineCommand {
    #[strum(serialize = "create-order")]
    CreateOrder(CreateOrderParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderParams {
    #[serde(flatten)]
    pub base_params: BaseCommandParams,
    pub account_id: i32,
    pub exchange: Exchange,
    pub symbol: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub comment: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionDetailParams {
    pub strategy_id: i64,
    pub node_id: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub transaction_id: Option<i64>,
    pub position_id: Option<i64>,
    pub order_id: Option<i64>,
}

