use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::market::Exchange;
use uuid::Uuid;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum ExchangeEngineCommand {
    #[strum(serialize = "register-exchange")]
    RegisterExchange(RegisterExchangeParams),
    #[strum(serialize = "unregister-exchange")]
    UnregisterExchange(UnregisterExchangeParams),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub exchange: Exchange,
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub sender: String,
    pub timestamp: i64,
    pub request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMt5ExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub login: i64, // 账户id
    pub password: String, // 密码
    pub server: String, // 服务器
    pub terminal_path: String, // 终端路径
    pub sender: String, // 发送者
    pub timestamp: i64, // 时间戳
    pub request_id: Uuid, // 请求id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterMt5ExchangeParams {
    pub terminal_id: i32, // 终端id 和系统的account_config的id一致
    pub sender: String, // 发送者
    pub timestamp: i64, // 时间戳
    pub request_id: Uuid, // 请求id
}
