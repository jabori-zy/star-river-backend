pub mod mt5_account;

use crate::market::Exchange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeStatus {
    NotRegist,      // 未注册
    Registing,      // 注册中
    Registed,       // 已注册
    RegisterFailed, // 注册失败
    Error,          // 错误
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum Account {
//     Mt5Account(Mt5Account),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub account_config: AccountConfig,
    pub account_info: Option<AccountInfo>,
    pub exchange_status: ExchangeStatus,
}

impl Account {
    pub fn new(
        config: AccountConfig,
        info: Option<AccountInfo>,
        exchange_status: ExchangeStatus,
    ) -> Self {
        Self {
            account_config: config,
            account_info: info,
            exchange_status,
        }
    }

    pub fn get_account_id(&self) -> i32 {
        self.account_config.id
    }

    pub fn get_account_name(&self) -> String {
        self.account_config.account_name.clone()
    }

    pub fn get_exchange(&self) -> Exchange {
        self.account_config.exchange.clone()
    }

    pub fn get_is_available(&self) -> bool {
        self.account_config.is_available
    }

    pub fn get_account_config(&self) -> AccountConfig {
        self.account_config.clone()
    }

    pub fn get_account_info(&self) -> Option<AccountInfo> {
        self.account_info.clone()
    }

    pub fn get_exchange_status(&self) -> ExchangeStatus {
        self.exchange_status.clone()
    }

    pub fn set_exchange_status(&mut self, status: ExchangeStatus) {
        self.exchange_status = status;
    }

    pub fn set_account_info(&mut self, account_info: AccountInfo) {
        self.account_info = Some(account_info);
    }

    pub fn set_account_config(&mut self, account_config: AccountConfig) {
        self.account_config = account_config;
    }
}

//系统的账户配置
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountConfig {
    pub id: i32,                    // 账户id
    pub account_name: String,       // 账户名称
    pub exchange: Exchange,         // 交易所
    pub config: serde_json::Value,  // 账户配置
    pub is_available: bool,         // 是否可用
    pub is_deleted: bool,           // 是否删除
    pub sort_index: i32,            // 排序索引
    pub create_time: DateTime<Utc>, // 创建时间
    pub update_time: DateTime<Utc>, // 更新时间
}

// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: i32,
    pub account_id: i32,            // 配置id
    pub info: serde_json::Value,    // 账户信息
    pub create_time: DateTime<Utc>, // 创建时间
    pub update_time: DateTime<Utc>, // 更新时间
}

// 原始账户
pub trait AccountTrait: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn AccountTrait>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_account_id(&self) -> i32; // 获取账户id
    fn get_account_name(&self) -> String; // 获取账户名称
    fn get_exchange(&self) -> Exchange; // 获取交易所
    fn get_is_available(&self) -> bool; // 获取是否可用
    fn get_account_config(&self) -> AccountConfig; // 获取账户配置
    fn get_account_info(&self) -> Option<AccountInfo>; // 获取账户信息
    fn get_exchange_status(&self) -> ExchangeStatus; // 获取交易所状态
    fn set_exchange_status(&mut self, status: ExchangeStatus); // 设置交易所状态
    fn set_account_info(&mut self, account_info: AccountInfo); // 设置账户信息
}

impl Clone for Box<dyn AccountTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// binance 账户配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinanceAccountConfig {
    pub api_key: String,
    pub api_secret: String,
}

pub trait ExchangeAccountConfig: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn ExchangeAccountConfig>;
    fn as_any(&self) -> &dyn Any;
    fn get_account_id(&self) -> i32; // 获取账户id
    fn get_account_name(&self) -> String; // 获取账户名称
    fn get_exchange(&self) -> Exchange; // 获取交易所
    fn get_is_available(&self) -> bool; // 获取是否可用
}

impl Clone for Box<dyn ExchangeAccountConfig> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait OriginalAccountInfo: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn OriginalAccountInfo>;
    fn as_any(&self) -> &dyn Any;
    fn get_account_id(&self) -> i32; // 获取账户id
    fn to_json(&self) -> serde_json::Value; // 转换为json
}

impl Clone for Box<dyn OriginalAccountInfo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
