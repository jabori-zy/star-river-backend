use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::custom_type::*;
use crate::market::Exchange;
use crate::transaction::TransactionType;
use crate::transaction::TransactionSide;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTransaction {
    pub transaction_id: i32,
    pub symbol: String,
    pub transaction_type: TransactionType,
    pub transaction_side: TransactionSide,
    pub quantity: f64,
    pub price: f64,
    pub create_time: DateTime<Utc>,
}