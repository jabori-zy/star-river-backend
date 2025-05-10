use serde::{Deserialize, Serialize};
use types::{cache::cache_key::{IndicatorCacheKey, KlineCacheKey}, custom_type::{StrategyId, NodeId}};
use strum::Display;
use uuid::Uuid;
use types::cache::{CacheValue, CacheKey};
use types::market::Kline;
use std::sync::Arc;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum CacheEngineResponse {
    SubscribedIndicator(SubscribedIndicatorResponse),
    AddIndicatorCacheKey(AddIndicatorCacheKeyResponse),
    GetCacheData(GetCacheDataResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribedIndicatorResponse {
    pub indicator_cache_key_list: Vec<IndicatorCacheKey>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCacheDataResponse {
    pub code: i32,
    pub message: String,
    pub cache_key: CacheKey,
    pub cache_data: Vec<Arc<CacheValue>>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddIndicatorCacheKeyResponse {
    pub code: i32,
    pub message: String,
    pub requested_strategy_id: StrategyId, // 请求的策略id
    pub requested_node_id: NodeId, // 请求的节点id
    pub indicator_cache_key: CacheKey,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}



