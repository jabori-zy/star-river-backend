pub mod mt5_account;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt::Debug;
use crate::market::Exchange;
use crate::account::mt5_account::Mt5Account;
use crate::account::mt5_account::Mt5AccountConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeStatus {
    NotRegist, // 未注册
    Registing, // 注册中
    Registed, // 已注册
    RegisterFailed, // 注册失败
    Error, // 错误
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Account {
    Mt5Account(Mt5Account),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AccountConfig {
    MetaTrader5(Mt5AccountConfig),
}


pub trait AccountTrait : Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn AccountTrait>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_account_id(&self) -> i32; // 获取账户id
    fn get_account_name(&self) -> String; // 获取账户名称
    fn get_exchange(&self) -> Exchange; // 获取交易所
    fn get_is_available(&self) -> bool; // 获取是否可用
    fn get_account_config(&self) -> Box<dyn ExchangeAccountConfig>; // 获取账户配置
    fn get_account_info(&self) -> Option<Box<dyn ExchangeAccountInfo>>; // 获取账户信息
    fn get_exchange_status(&self) -> ExchangeStatus; // 获取交易所状态
    fn set_exchange_status(&mut self, status: ExchangeStatus); // 设置交易所状态
    fn set_account_info(&mut self, account_info: Box<dyn ExchangeAccountInfo>); // 设置账户信息
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

pub trait ExchangeAccountInfo: Debug + Send + Sync + Any + 'static {
    fn clone_box(&self) -> Box<dyn ExchangeAccountInfo>;
    fn as_any(&self) -> &dyn Any;
    fn get_account_id(&self) -> i32; // 获取账户id
}


impl Clone for Box<dyn ExchangeAccountInfo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}





