pub mod virtual_transaction;


use serde::{Serialize, Deserialize};
use crate::market::Exchange;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt::Debug;
use strum::{EnumString, Display};

// 交易明细类型
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum TransactionType {
    #[strum(serialize = "open")]
    Open, // 开仓
    #[strum(serialize = "close")]
    Close, // 平仓
}


// 交易方向
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
pub enum TransactionSide {
    #[strum(serialize = "long")]
    Long, // 多头
    #[strum(serialize = "short")]
    Short, // 空头
}



//交易明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: i32,
    pub symbol: String,
    pub exchange: Exchange,
    pub exchange_order_id: i64,
    pub exchange_position_id: i64,
    pub exchange_transaction_id: i64,
    pub transaction_type: TransactionType,
    pub transaction_side: TransactionSide,
    pub quantity: f64,
    pub price: f64,
    pub create_time: DateTime<Utc>,
    pub extra_info: Option<serde_json::Value>, // 额外信息
}


pub trait OriginalTransaction : Debug +Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OriginalTransaction>;
    fn get_transaction_id(&self) -> i64;
    fn get_transaction_type(&self) -> TransactionType;
    fn get_transaction_side(&self) -> TransactionSide;
    fn get_quantity(&self) -> f64;
    fn get_price(&self) -> f64;
    fn get_create_time(&self) -> DateTime<Utc>;
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


