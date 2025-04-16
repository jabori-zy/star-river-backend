use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt::Debug;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: i32,
    pub account_name: String,
    pub exchange: String,
    pub is_available: bool,
    pub account_config: Value,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
}

pub trait ExchangeAccountInfo: Debug + Send + Sync + Any + 'static {
    fn get_account_id(&self) -> i64; // 获取账户id
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mt5AccountInfo {
    pub id: i32,
    pub account_id: i64,
    pub trade_mode: String,
    pub leverage: i64,
    pub limit_orders: i64,
    pub margin_stopout_mode: String,
    pub trade_allowed: bool,
    pub trade_expert: bool,
    pub margin_mode: String,
    pub currency_digits: i64,
    pub fifo_close: bool,
    pub balance: f64,
    pub credit: f64,
    pub profit: f64,
    pub equity: f64,
    pub margin: f64,
    pub margin_free: f64,
    pub margin_level: f64,
    pub margin_so_call: f64,
    pub margin_so_so: f64,
    pub margin_initial: f64,
    pub margin_maintenance: f64,
    pub assets: f64,
    pub liabilities: f64,
    pub commission_blocked: f64,
    pub name: String,
    pub server: String,
    pub currency: String,
    pub company: String,
}

impl ExchangeAccountInfo for Mt5AccountInfo {
    fn get_account_id(&self) -> i64 {
        self.account_id
    }
}

