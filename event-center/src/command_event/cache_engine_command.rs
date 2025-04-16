use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::cache::{KlineCacheKey, IndicatorCacheKey};
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineCommand {
    #[strum(serialize = "add-kline-cache-key")]
    AddKlineCacheKey(AddKlineCacheKeyParams), // 添加K线缓存键
    #[strum(serialize = "subscribe-indicator")]
    SubscribeIndicator(SubscribeIndicatorParams), // 订阅指标
    #[strum(serialize = "get-subscribed-indicator")]
    GetSubscribedIndicator(GetSubscribedIndicatorParams), // 获取订阅的指标
}

// 添加K线缓存键参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKlineCacheKeyParams {
    pub strategy_id: i64,
    pub cache_key: KlineCacheKey,
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