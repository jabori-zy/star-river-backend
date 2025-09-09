use super::CommandTrait;
use crate::command::Command;
use crate::Responder;
use std::fmt::Debug;
use tokio::time::Duration;
use types::cache::Key;
use types::custom_type::{NodeId, StrategyId};
use types::market::{Exchange, KlineInterval};

#[derive(Debug)]
pub enum CacheEngineCommand {
    AddCacheKey(AddCacheKeyParams),                 // 添加缓存键
    GetCache(GetCacheParams),                       // 获取缓存数据
    GetCacheMulti(GetCacheMultiParams),             // 一次性获取多个key的数据
    GetCacheLength(GetCacheLengthParams),           // 获取缓存长度
    GetCacheLengthMulti(GetCacheLengthMultiParams), // 一次性获取多个key的缓存长度
}

impl CommandTrait for CacheEngineCommand {
    fn responder(&self) -> &Responder {
        match self {
            CacheEngineCommand::AddCacheKey(params) => &params.responder,
            CacheEngineCommand::GetCache(params) => &params.responder,
            CacheEngineCommand::GetCacheMulti(params) => &params.responder,
            CacheEngineCommand::GetCacheLength(params) => &params.responder,
            CacheEngineCommand::GetCacheLengthMulti(params) => &params.responder,
        }
    }
    fn timestamp(&self) -> i64 {
        match self {
            CacheEngineCommand::AddCacheKey(params) => params.timestamp,
            CacheEngineCommand::GetCache(params) => params.timestamp,
            CacheEngineCommand::GetCacheMulti(params) => params.timestamp,
            CacheEngineCommand::GetCacheLength(params) => params.timestamp,
            CacheEngineCommand::GetCacheLengthMulti(params) => params.timestamp,
        }
    }

    fn sender(&self) -> String {
        match self {
            CacheEngineCommand::AddCacheKey(params) => params.sender.clone(),
            CacheEngineCommand::GetCache(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheMulti(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheLength(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheLengthMulti(params) => params.sender.clone(),
        }
    }
}

impl From<CacheEngineCommand> for Command {
    fn from(command: CacheEngineCommand) -> Self {
        Command::CacheEngine(command)
    }
}

// 添加K线缓存键参数
#[derive(Debug)]
pub struct AddCacheKeyParams {
    pub strategy_id: StrategyId,
    pub key: Key,
    pub max_size: Option<u32>,
    pub duration: Duration,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

// 添加指标缓存键参数
#[derive(Debug)]
pub struct AddIndicatorCacheKeyParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_cache_key: Key,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct SubscribeIndicatorParams {
    pub cache_key: Key,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct GetSubscribedIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct GetCacheParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub key: Key,           // 缓存键
    pub index: Option<u32>, // 缓存取值索引
    pub limit: Option<u32>, // 获取的缓存数据条数
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct GetCacheMultiParams {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
    pub index: Option<u32>, // 缓存取值索引
    pub limit: Option<u32>,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct GetCacheLengthParams {
    pub strategy_id: StrategyId,
    pub cache_key: Key,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}

#[derive(Debug)]
pub struct GetCacheLengthMultiParams {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
    pub sender: String,
    pub timestamp: i64,
    pub responder: Responder,
}
