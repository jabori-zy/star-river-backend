
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::any::Any;
use crate::account::{ExchangeAccountInfo,ExchangeAccountConfig};
use crate::market::Exchange;
use std::str::FromStr;


// metatrader5 账户配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mt5AccountConfig {
    pub id: i32, // 账户id
    pub account_name: String, // 账户名称
    pub exchange: String, // 交易所
    pub is_available: bool, // 是否可用
    pub login: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
    pub sort_index: i32,
    pub created_time: DateTime<Utc>, // 创建时间
    pub updated_time: DateTime<Utc>, // 更新时间
}


impl ExchangeAccountConfig for Mt5AccountConfig {
    fn clone_box(&self) -> Box<dyn ExchangeAccountConfig> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_account_id(&self) -> i32 {
        self.id
    }

    fn get_account_name(&self) -> String {
        self.account_name.clone()
    }

    fn get_exchange(&self) -> Exchange {
        Exchange::from_str(self.exchange.as_str()).unwrap()
    }

    fn get_is_available(&self) -> bool {
        self.is_available
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mt5AccountInfo {
    pub terminal_id: i64,
    pub login: i64,
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
    fn clone_box(&self) -> Box<dyn ExchangeAccountInfo> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_account_id(&self) -> i64 {
        self.terminal_id
    }
}