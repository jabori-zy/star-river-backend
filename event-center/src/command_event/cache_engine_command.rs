use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::cache::{KlineCacheKey, IndicatorCacheKey};
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use types::new_cache::CacheKey;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineCommand {
    #[strum(serialize = "add-cache-key")]
    AddCacheKey(AddCacheKeyParams), // 添加缓存键
    #[strum(serialize = "subscribe-indicator")]
    SubscribeIndicator(SubscribeIndicatorParams), // 订阅指标
    #[strum(serialize = "get-subscribed-indicator")]
    GetSubscribedIndicator(GetSubscribedIndicatorParams), // 获取订阅的指标
    #[strum(serialize = "get-cache-data")]
    GetCacheData(GetCacheDataParams), // 获取缓存数据
}

// 添加K线缓存键参数
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AddCacheKeyParams {
    pub strategy_id: i32,
    pub cache_key: CacheKey,
    pub max_size: u32,
    pub sender: String,
    pub timestamp:i64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeIndicatorParams {
    pub cache_key: IndicatorCacheKey,
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
pub struct GetCacheDataParams {
    pub strategy_id: i32,
    pub cache_key: CacheKey, // 缓存键
    pub limit: u32, // 获取的缓存数据条数
    pub sender: String,
    pub request_id: Uuid,
    pub timestamp:i64,
}

