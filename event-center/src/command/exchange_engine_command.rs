use std::fmt::Debug;
use types::market::Exchange;
use crate::command::Command;
use crate::{Responder};
use super::CommandTrait;

#[derive(Debug)]
pub enum ExchangeEngineCommand {
    RegisterExchange(RegisterExchangeParams),
    UnregisterExchange(UnregisterExchangeParams),
}

impl CommandTrait for ExchangeEngineCommand {
    fn responder(&self) -> &Responder {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => &params.responder,
            ExchangeEngineCommand::UnregisterExchange(params) => &params.responder,
        }
    }

    fn timestamp(&self) -> i64 {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => params.timestamp,
            ExchangeEngineCommand::UnregisterExchange(params) => params.timestamp,
        }
    }

    fn sender(&self) -> String {
        match self {
            ExchangeEngineCommand::RegisterExchange(params) => params.sender.clone(),
            ExchangeEngineCommand::UnregisterExchange(params) => params.sender.clone(),
        }
    }
    
}
impl From<ExchangeEngineCommand> for Command {
    fn from(command: ExchangeEngineCommand) -> Self {
        Command::ExchangeEngine(command)
    }
}


#[derive(Debug)]
pub struct RegisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub exchange: Exchange,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct UnregisterExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct RegisterMt5ExchangeParams {
    pub account_id: i32, // 终端id 和系统的account_config的id一致
    pub login: i64, // 账户id
    pub password: String, // 密码
    pub server: String, // 服务器
    pub terminal_path: String, // 终端路径
    pub sender: String, // 发送者
    pub timestamp: i64, // 时间戳
}

#[derive(Debug)]
pub struct UnregisterMt5ExchangeParams {
    pub terminal_id: i32, // 终端id 和系统的account_config的id一致
    pub sender: String, // 发送者
    pub timestamp: i64, // 时间戳
}
