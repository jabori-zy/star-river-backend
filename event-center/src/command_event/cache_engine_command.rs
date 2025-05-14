use serde::{Deserialize, Serialize};
use strum::Display;
use types::custom_type::{NodeId, StrategyId};
use std::fmt::Debug;
use types::cache::CacheKey;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineCommand {
    #[strum(serialize = "add-cache-key")]
    AddCacheKey(AddCacheKeyParams), // 添加缓存键
    #[strum(serialize = "get-cache")]
    GetCache(GetCacheParams), // 获取缓存数据
    #[strum(serialize = "get-cache-multi")]
    GetCacheMulti(GetCacheMultiParams), // 一次性获取多个key的数据
}

// 添加K线缓存键参数
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AddCacheKeyParams {
    pub strategy_id: StrategyId,
    pub cache_key: CacheKey,
    pub max_size: Option<u32>,
    pub duration: Duration,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}

// 添加指标缓存键参数
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AddIndicatorCacheKeyParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_cache_key: CacheKey,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeIndicatorParams {
    pub cache_key: CacheKey,
    pub sender: String,
    pub timestamp:i64,
}


#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct GetSubscribedIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: String,
    pub timestamp:i64,
    pub request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub cache_key: CacheKey, // 缓存键
    pub limit: Option<u32>, // 获取的缓存数据条数
    pub sender: String,
    pub request_id: Uuid,
    pub timestamp:i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheMultiParams {
    pub strategy_id: StrategyId,
    pub cache_keys: Vec<CacheKey>,
    pub limit: Option<u32>,
    pub sender: String,
    pub request_id: Uuid,
    pub timestamp:i64,
}




