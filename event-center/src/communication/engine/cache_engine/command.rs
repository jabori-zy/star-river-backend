use super::super::{EngineCommand, EngineCommandTrait, EngineResponder};
use chrono::Utc;
use star_river_core::cache::{CacheValue, Key};
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::market::{Exchange, KlineInterval};
use star_river_core::system::DateTimeUtc;
use std::fmt::Debug;
use tokio::time::Duration;

#[derive(Debug)]
pub enum CacheEngineCommand {
    AddCacheKey(AddCacheKeyParams),                 // 添加缓存键
    GetCache(GetCacheParams),                       // 获取缓存数据
    GetCacheMulti(GetCacheMultiParams),             // 一次性获取多个key的数据
    GetCacheLength(GetCacheLengthParams),           // 获取缓存长度
    GetCacheLengthMulti(GetCacheLengthMultiParams), // 一次性获取多个key的缓存长度
    UpdateCache(UpdateCacheParams),                 // 更新缓存数据
    ClearCache(ClearCacheParams),                   // 清空缓存
}

impl EngineCommandTrait for CacheEngineCommand {
    fn responder(&self) -> &EngineResponder {
        match self {
            CacheEngineCommand::AddCacheKey(params) => &params.responder,
            CacheEngineCommand::GetCache(params) => &params.responder,
            CacheEngineCommand::GetCacheMulti(params) => &params.responder,
            CacheEngineCommand::GetCacheLength(params) => &params.responder,
            CacheEngineCommand::GetCacheLengthMulti(params) => &params.responder,
            CacheEngineCommand::UpdateCache(params) => &params.responder,
            CacheEngineCommand::ClearCache(params) => &params.responder,
        }
    }
    fn datetime(&self) -> DateTimeUtc {
        match self {
            CacheEngineCommand::AddCacheKey(params) => params.datetime,
            CacheEngineCommand::GetCache(params) => params.datetime,
            CacheEngineCommand::GetCacheMulti(params) => params.datetime,
            CacheEngineCommand::GetCacheLength(params) => params.datetime,
            CacheEngineCommand::GetCacheLengthMulti(params) => params.datetime,
            CacheEngineCommand::UpdateCache(params) => params.datetime,
            CacheEngineCommand::ClearCache(params) => params.datetime,
        }
    }

    fn sender(&self) -> String {
        match self {
            CacheEngineCommand::AddCacheKey(params) => params.sender.clone(),
            CacheEngineCommand::GetCache(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheMulti(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheLength(params) => params.sender.clone(),
            CacheEngineCommand::GetCacheLengthMulti(params) => params.sender.clone(),
            CacheEngineCommand::UpdateCache(params) => params.sender.clone(),
            CacheEngineCommand::ClearCache(params) => params.sender.clone(),
        }
    }
}

impl From<CacheEngineCommand> for EngineCommand {
    fn from(command: CacheEngineCommand) -> Self {
        EngineCommand::CacheEngine(command)
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
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl AddCacheKeyParams {
    pub fn new(
        strategy_id: StrategyId,
        key: Key,
        max_size: Option<u32>,
        duration: Duration,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            key,
            max_size,
            duration,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<AddCacheKeyParams> for EngineCommand {
    fn from(params: AddCacheKeyParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::AddCacheKey(params))
    }
}

// 添加指标缓存键参数
#[derive(Debug)]
pub struct AddIndicatorCacheKeyParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_cache_key: Key,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl AddIndicatorCacheKeyParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        indicator_cache_key: Key,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            indicator_cache_key,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct SubscribeIndicatorParams {
    pub cache_key: Key,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl SubscribeIndicatorParams {
    pub fn new(cache_key: Key, sender: String, responder: EngineResponder) -> Self {
        Self {
            cache_key,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct GetSubscribedIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetSubscribedIndicatorParams {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct GetCacheParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub key: Key,           // 缓存键
    pub index: Option<u32>, // 缓存取值索引
    pub limit: Option<u32>, // 获取的缓存数据条数
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetCacheParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        key: Key,
        index: Option<u32>,
        limit: Option<u32>,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            key,
            index,
            limit,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetCacheParams> for EngineCommand {
    fn from(params: GetCacheParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::GetCache(params))
    }
}

#[derive(Debug)]
pub struct GetCacheMultiParams {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
    pub index: Option<u32>, // 缓存取值索引
    pub limit: Option<u32>,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetCacheMultiParams {
    pub fn new(
        strategy_id: StrategyId,
        keys: Vec<Key>,
        index: Option<u32>,
        limit: Option<u32>,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            keys,
            index,
            limit,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetCacheMultiParams> for EngineCommand {
    fn from(params: GetCacheMultiParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::GetCacheMulti(params))
    }
}

#[derive(Debug)]
pub struct GetCacheLengthParams {
    pub strategy_id: StrategyId,
    pub cache_key: Key,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetCacheLengthParams {
    pub fn new(strategy_id: StrategyId, cache_key: Key, sender: String, responder: EngineResponder) -> Self {
        Self {
            strategy_id,
            cache_key,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct GetCacheLengthMultiParams {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetCacheLengthMultiParams {
    pub fn new(strategy_id: StrategyId, keys: Vec<Key>, sender: String, responder: EngineResponder) -> Self {
        Self {
            strategy_id,
            keys,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetCacheLengthMultiParams> for EngineCommand {
    fn from(params: GetCacheLengthMultiParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::GetCacheLengthMulti(params))
    }
}

#[derive(Debug)]
pub struct UpdateCacheParams {
    pub strategy_id: StrategyId,
    pub key: Key,
    pub cache_value: CacheValue,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl UpdateCacheParams {
    pub fn new(
        strategy_id: StrategyId,
        key: Key,
        cache_value: CacheValue,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            key,
            cache_value,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<UpdateCacheParams> for EngineCommand {
    fn from(params: UpdateCacheParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::UpdateCache(params))
    }
}

#[derive(Debug)]
pub struct ClearCacheParams {
    pub strategy_id: StrategyId,
    pub key: Key,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl ClearCacheParams {
    pub fn new(strategy_id: StrategyId, key: Key, sender: String, responder: EngineResponder) -> Self {
        Self {
            strategy_id,
            key,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<ClearCacheParams> for EngineCommand {
    fn from(params: ClearCacheParams) -> Self {
        EngineCommand::CacheEngine(CacheEngineCommand::ClearCache(params))
    }
}
