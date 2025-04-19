pub mod mt5_account;


use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::fmt::Debug;
use crate::market::Exchange;

// #[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ExchangeAccountConfig {
//     MetaTrader5(Mt5AccountConfig),
//     Binance(BinanceAccountConfig),
// }


// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct AccountConfig {
//     pub id: i32, // 账户id
//     pub account_name: String, // 账户名称
//     pub exchange: Exchange, // 交易所
//     pub is_available: bool, // 是否可用
//     pub account_config: ExchangeAccountConfig, // 账户配置
//     pub created_time: DateTime<Utc>, // 创建时间
//     pub updated_time: DateTime<Utc>, // 更新时间
// }




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
    fn get_account_id(&self) -> i64; // 获取账户id
}


impl Clone for Box<dyn ExchangeAccountInfo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}





