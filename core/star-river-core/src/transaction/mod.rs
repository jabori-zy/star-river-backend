// pub mod virtual_transaction;

use std::{any::Any, fmt::Debug, str::FromStr};

use entity::transaction::Model as TransactionModel;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::{exchange::Exchange, order::FuturesOrderSide, system::DateTimeUtc};

// Transaction type
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display, ToSchema, PartialEq, Eq)]
pub enum TransactionType {
    #[strum(serialize = "OPEN")]
    Open, // Open position
    #[strum(serialize = "CLOSE")]
    Close, // Close position
}

// Transaction side
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display, ToSchema)]
pub enum FuturesTransSide {
    #[strum(serialize = "LONG")]
    Long, // Long
    #[strum(serialize = "SHORT")]
    Short, // Short
}

impl From<FuturesOrderSide> for FuturesTransSide {
    fn from(order_side: FuturesOrderSide) -> Self {
        match order_side {
            FuturesOrderSide::Long => FuturesTransSide::Long,
            FuturesOrderSide::Short => FuturesTransSide::Short,
        }
    }
}

// Transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: i32,
    pub symbol: String,
    pub exchange: Exchange,
    pub exchange_order_id: i64,
    pub exchange_position_id: i64,
    pub exchange_transaction_id: i64,
    pub transaction_type: TransactionType,
    pub transaction_side: FuturesTransSide,
    pub quantity: f64,
    pub price: f64,
    pub create_time: DateTimeUtc,
    pub extra_info: Option<serde_json::Value>, // Extra info
}

impl From<TransactionModel> for Transaction {
    fn from(model: TransactionModel) -> Self {
        Self {
            transaction_id: model.id,
            symbol: model.symbol,
            exchange: Exchange::from_str(&model.exchange).unwrap(),
            exchange_order_id: model.exchange_order_id,
            exchange_position_id: model.exchange_position_id,
            exchange_transaction_id: model.exchange_transaction_id,
            transaction_type: TransactionType::from_str(&model.transaction_type).unwrap(),
            transaction_side: FuturesTransSide::from_str(&model.transaction_side).unwrap(),
            quantity: model.quantity,
            price: model.price,
            create_time: model.created_time,
            extra_info: model.extra_info,
        }
    }
}

pub trait OriginalTransaction: Debug + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalTransaction>;
    fn get_transaction_id(&self) -> i64;
    fn get_transaction_type(&self) -> TransactionType;
    fn get_transaction_side(&self) -> FuturesTransSide;
    fn get_quantity(&self) -> f64;
    fn get_price(&self) -> f64;
    fn get_create_time(&self) -> DateTimeUtc;
    fn get_symbol(&self) -> String;
    fn get_exchange(&self) -> Exchange;
    fn get_exchange_order_id(&self) -> i64;
    fn get_exchange_position_id(&self) -> i64;
    fn get_exchange_transaction_id(&self) -> i64;
}

impl Clone for Box<dyn OriginalTransaction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
