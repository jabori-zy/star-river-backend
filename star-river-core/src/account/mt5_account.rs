use crate::account::AccountConfig;
use crate::account::AccountInfo;
use crate::account::AccountTrait;
use crate::account::ExchangeStatus;
use crate::account::OriginalAccountInfo;
use crate::market::Exchange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::str::FromStr;

// metatrader5 账户配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mt5AccountConfig {
    pub id: i32,              // 账户id
    pub account_name: String, // 账户名称
    pub exchange: Exchange,   // 交易所
    pub login: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
    pub is_available: bool, // 是否可用
    pub sort_index: i32,
    pub create_time: DateTime<Utc>, // 创建时间
    pub update_time: DateTime<Utc>, // 更新时间
}

// 将AccountConfig转换为Mt5AccountConfig
impl From<AccountConfig> for Mt5AccountConfig {
    fn from(account_config: AccountConfig) -> Self {
        Mt5AccountConfig {
            id: account_config.id,
            account_name: account_config.account_name,
            exchange: account_config.exchange,
            login: account_config.config["login"].as_i64().unwrap(),
            password: account_config.config["password"]
                .as_str()
                .unwrap()
                .to_string(),
            server: account_config.config["server"]
                .as_str()
                .unwrap()
                .to_string(),
            terminal_path: account_config.config["terminal_path"]
                .as_str()
                .unwrap()
                .to_string(),
            is_available: account_config.is_available,
            sort_index: account_config.sort_index,
            create_time: account_config.create_time,
            update_time: account_config.update_time,
        }
    }
}

// 原始mt5账户信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OriginalMt5AccountInfo {
    pub account_id: i32,
    pub login: i64,
    pub trade_mode: String,
    pub leverage: i64,
    pub limit_orders: i64,
    pub margin_stopout_mode: String,
    pub trade_allowed: bool,
    pub dlls_allowed: bool,
    pub terminal_connected: bool,
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

impl OriginalAccountInfo for OriginalMt5AccountInfo {
    fn clone_box(&self) -> Box<dyn OriginalAccountInfo> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_account_id(&self) -> i32 {
        self.account_id
    }
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

// 处理后的mt5账户信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mt5AccountInfo {
    pub id: i32,
    pub account_id: i32,
    pub login: i64,
    pub trade_mode: String,
    pub leverage: i64,
    pub limit_orders: i64,
    pub margin_stopout_mode: String,
    pub trade_allowed: bool,
    pub dlls_allowed: bool,
    pub terminal_connected: bool,
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
    pub create_time: DateTime<Utc>, // 创建时间
    pub update_time: DateTime<Utc>, // 更新时间
}

impl From<AccountInfo> for Mt5AccountInfo {
    fn from(account_info: AccountInfo) -> Self {
        Mt5AccountInfo {
            id: account_info.id,
            account_id: account_info.account_id,
            login: account_info.info["login"].as_i64().unwrap(),
            trade_mode: account_info.info["trade_mode"]
                .as_str()
                .unwrap()
                .to_string(),
            leverage: account_info.info["leverage"].as_i64().unwrap(),
            limit_orders: account_info.info["limit_orders"].as_i64().unwrap(),
            margin_stopout_mode: account_info.info["margin_stopout_mode"]
                .as_str()
                .unwrap()
                .to_string(),
            trade_allowed: account_info.info["trade_allowed"].as_bool().unwrap(),
            dlls_allowed: account_info.info["dlls_allowed"].as_bool().unwrap(),
            terminal_connected: account_info.info["terminal_connected"].as_bool().unwrap(),
            trade_expert: account_info.info["trade_expert"].as_bool().unwrap(),
            margin_mode: account_info.info["margin_mode"]
                .as_str()
                .unwrap()
                .to_string(),
            currency_digits: account_info.info["currency_digits"].as_i64().unwrap(),
            fifo_close: account_info.info["fifo_close"].as_bool().unwrap(),
            balance: account_info.info["balance"].as_f64().unwrap(),
            credit: account_info.info["credit"].as_f64().unwrap(),
            profit: account_info.info["profit"].as_f64().unwrap(),
            equity: account_info.info["equity"].as_f64().unwrap(),
            margin: account_info.info["margin"].as_f64().unwrap(),
            margin_free: account_info.info["margin_free"].as_f64().unwrap(),
            margin_level: account_info.info["margin_level"].as_f64().unwrap(),
            margin_so_call: account_info.info["margin_so_call"].as_f64().unwrap(),
            margin_so_so: account_info.info["margin_so_so"].as_f64().unwrap(),
            margin_initial: account_info.info["margin_initial"].as_f64().unwrap(),
            margin_maintenance: account_info.info["margin_maintenance"].as_f64().unwrap(),
            assets: account_info.info["assets"].as_f64().unwrap(),
            liabilities: account_info.info["liabilities"].as_f64().unwrap(),
            commission_blocked: account_info.info["commission_blocked"].as_f64().unwrap(),
            name: account_info.info["name"].as_str().unwrap().to_string(),
            server: account_info.info["server"].as_str().unwrap().to_string(),
            currency: account_info.info["currency"].as_str().unwrap().to_string(),
            company: account_info.info["company"].as_str().unwrap().to_string(),
            create_time: account_info.create_time,
            update_time: account_info.update_time,
        }
    }
}
